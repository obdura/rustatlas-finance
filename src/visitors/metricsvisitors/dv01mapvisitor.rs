use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::prelude::{Currency, FixingVisitor, HasReferenceDate, NPVConstVisitor, SimpleModel, Visit};
use crate::{
    core::marketstore::MarketStore,
    models::traits::Model,
    rates::{
        yieldtermstructure::discounttermstructure::DiscountTermStructure,
        interestrateindex::iborindex::IborIndex, // Asume que los índices son IborIndex; ajusta si no
    },
    time::{date::Date, daycounter::DayCounter},
    utils::errors::{AtlasError, Result},
    visitors::traits::{ConstVisit, HasCashflows},
    math::interpolation::enums::Interpolator,
};

/// # DV01MapVisitor
/// Visitor que calcula un mapa de DV01 mediante perturbación numérica.
/// Itera sobre los pilares existentes en las curvas especificadas.
/// Para cada combinación de curva y pilar, perturba la tasa en +1 bp, recalcula NPV y mide el delta.
///
/// ## Parámetros
/// * `base_market_store` - MarketStore base (se clona para perturbaciones)
/// * `curve_ids` - Lista de IDs de curvas a perturbar
/// * `perturbation_bp` - Perturbación en puntos base (default 1.0)
pub struct DV01MapVisitor {
    base_market_store: MarketStore,
    curve_ids: Vec<usize>,
    perturbation_bp: f64,
}

impl DV01MapVisitor {
    pub fn new(base_market_store: MarketStore) -> Self {
        Self {
            base_market_store,
            curve_ids: Vec::new(),
            perturbation_bp: 1.0,
        }
    }

    pub fn with_curve_ids(mut self, curve_ids: Vec<usize>) -> Self {
        self.curve_ids = curve_ids;
        self
    }

    pub fn with_perturbation_bp(mut self, bp: f64) -> Self {
        self.perturbation_bp = bp;
        self
    }

    /// Crea un HashMap<(curve_id, Date), MarketStore> con stores perturbados por cada pilar existente en las curvas.
    /// Cada store tiene solo una curva perturbada en un pilar.
    pub fn create_perturbed_stores(&self) -> Result<HashMap<(usize, Date), MarketStore>> {
        let mut perturbed_stores = HashMap::new();
        let ref_date = self.base_market_store.reference_date();

        for &curve_id in &self.curve_ids {
            // Obtener los pilares de la curva
            let existing_index_arc = self.base_market_store.get_index(curve_id)?;
            let existing_index_guard = existing_index_arc.read().map_err(|_| {
                AtlasError::InvalidValueErr("Poisoned lock al leer índice existente".to_string())
            })?;
            let current_ts = existing_index_guard.term_structure()?;
            let discount_ts = current_ts.as_any().downcast_ref::<DiscountTermStructure>().ok_or_else(|| {
                AtlasError::InvalidValueErr(format!("Curva {} no es DiscountTermStructure", curve_id))
            })?;
            let dates = discount_ts.dates().clone();

            for &pillar_date in &dates {
                let mut perturbed_store = self.base_market_store.clone();
                self.perturb_curve_at_tenor(&mut perturbed_store, curve_id, pillar_date, ref_date)?;
                perturbed_stores.insert((curve_id, pillar_date), perturbed_store);
            }
        }

        Ok(perturbed_stores)
    }

    /// Método para calcular mapa de riesgo usando SimpleModel
    /// El instrument debe estar SIN FIXING (antes de aplicar FixingVisitor).
    /// Se clona y fija para cada modelo (base y perturbed).
    /// Retorna HashMap<(curve_id, Date), f64> con delta NPV por curva y pilar.
    pub fn calculate_risk_map(
        &self,
        instrument: &(impl HasCashflows + Clone),
    ) -> Result<HashMap<(usize, Date), f64>> {
        // Fija el instrumento base con base_model
        let base_model = SimpleModel::new(&self.base_market_store);
        let mut base_instrument = instrument.clone();
        let fixing_visitor_base = FixingVisitor::new(&base_model);
        fixing_visitor_base.visit(&mut base_instrument)?;
        let base_npv = self.calculate_npv(&base_model, &base_instrument)?;

        let perturbed_stores = self.create_perturbed_stores()?;
        let mut risk_map = HashMap::new();

        for ((curve_id, tenor_date), perturbed_store) in perturbed_stores {
            let perturbed_model = SimpleModel::new(&perturbed_store);
            let mut perturbed_instrument = instrument.clone();
            let fixing_visitor_perturbed = FixingVisitor::new(&perturbed_model);
            fixing_visitor_perturbed.visit(&mut perturbed_instrument)?;
            let perturbed_npv = self.calculate_npv(&perturbed_model, &perturbed_instrument)?;
            let delta_npv = perturbed_npv - base_npv;
            risk_map.insert((curve_id, tenor_date), delta_npv);
        }

        Ok(risk_map)
    }

    fn calculate_npv<M: Model>(
        &self,
        model: &M,
        instrument: &impl HasCashflows,
    ) -> Result<f64> {
        let npv_visitor = NPVConstVisitor::new(model, true);
        npv_visitor.visit(instrument)
    }

    fn perturb_curve_at_tenor(
        &self,
        market_store: &mut MarketStore,
        curve_id: usize,
        tenor: Date,
        ref_date: Date,
    ) -> Result<()> {
        let existing_index_arc = market_store.get_index(curve_id)?;
        let existing_index_guard = existing_index_arc.read().map_err(|_| {
            AtlasError::InvalidValueErr("Poisoned lock al leer índice existente".to_string())
        })?;

        let current_ts = existing_index_guard.term_structure()?;

        let df_actual = current_ts.discount_factor(tenor)?;
        let t = DayCounter::Actual360.year_fraction(ref_date, tenor);
        let r = -df_actual.ln() / t;
        let r_new = r + self.perturbation_bp * 0.0001; // 1 bp = 0.0001

        let discount_ts = current_ts.as_any().downcast_ref::<DiscountTermStructure>().ok_or_else(|| {
            AtlasError::InvalidValueErr(format!("Curva {} no es DiscountTermStructure", curve_id))
        })?;
        let mut dates = discount_ts.dates().clone();
        let mut dfs = discount_ts.discount_factors().clone();

        if let Some(idx) = dates.iter().position(|&d| d == tenor) {
            if idx == 0 {
                return Ok(());
            }
            let df_new = (-r_new * t).exp();
            dfs[idx] = df_new;
        } else {
            let closest_idx = dates.iter().enumerate()
                .min_by_key(|&(_, d)| ((*d) - tenor).abs())  // Agrega *d para desreferenciar
                .map(|(i, _)| i)
                .ok_or_else(|| AtlasError::InvalidValueErr("No hay pilares en la curva".to_string()))?;
            if closest_idx == 0 {
                return Ok(());
            }
            let df_new = (-r_new * t).exp();
            dfs[closest_idx] = df_new;
        }

        let new_ts = DiscountTermStructure::new(
            dates,
            dfs,
            DayCounter::Actual360,
            Interpolator::LogLinear,
            true,
        )?;

        let currency = market_store.mut_index_store().get_currency_of_index_id(curve_id)?.unwrap_or(Currency::USD);

        let new_index = IborIndex::new(ref_date)
            .with_term_structure(Arc::new(new_ts))
            .with_currency(Some(currency))
            .with_name(Some(existing_index_guard.name()?));

        market_store
            .mut_index_store()
            .replace_index(curve_id, Arc::new(RwLock::new(new_index)))?;

        Ok(())
    }
}

impl<T: HasCashflows> ConstVisit<T> for DV01MapVisitor {
    type Output = Result<HashMap<(usize, Date), MarketStore>>;

    fn visit(&self, _visitable: &T) -> Self::Output {
        self.create_perturbed_stores()
    }
}

