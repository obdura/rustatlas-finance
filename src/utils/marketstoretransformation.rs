use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::{
    core::marketstore::MarketStore,
    math::interpolation::enums::Interpolator,
    rates::{
        indexstore::ReadIndex,
        interestrate::RateDefinition,
        interestrateindex::traits::InterestRateIndexTrait,
        traits::HasReferenceDate,
        yieldtermstructure::{
            compositetermstructure::CompositeTermStructure, flatforwardtermstructure::FlatForwardTermStructure, tenorbasedspreadtermstructure::TenorBasedSpreadRateTermStructure, tenorbasedzeroratetermstructure::TenorBasedZeroRateTermStructure, zeroratetermstructure::ZeroRateTermStructure
        },
    },
    time::{date::Date, period::Period},
    utils::errors::{AtlasError, Result},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransformationType {
    ParallelShift, // apply a parallel shift to the curve -- shift is derived in forward rates -- forwards rate = fwr(base_curve + shift)
    TenorBasedShift, // apply a shift to the curve based using a tenor based term structure -- shift is derived in forward rates -- forwards rate = fwr(base_curve + shift)
    BaseAndSpread, // overwrite the curve using a base curve and a given spread curve -- spread is not derived in forward rates -- forwards rate = fwr(base_curve) + spread_curve
    ImplicitBaseAndSpread, // overwrite the curve using a two curves -- base curve and spread + base curve -- spread is not derived in forward rates -- forwards rate = fwr(base_curve) + spread_curve
    NewCurveAndSpread, // create a new curve using a base curve and a spread curve -- spread is not derived iq  n forward rates -- forwards rate = fwr(base_curve) + spread_curve
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TenorBasedValues {
    pub tenor: Period,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurveTransformations {
    pub apply_to: String,
    pub transformation_type: TransformationType,
    pub shift_value: Option<f64>,
    pub rate_definition: RateDefinition,
    pub shift_term_structure: Option<Vec<TenorBasedValues>>,
    pub base_term_structure: Option<String>,
    pub spread_term_structure: Option<Vec<TenorBasedValues>>,
}

/// # Apply Curve Transformations
/// This function applies a list of curve transformations to a market store curve and returns a new market store with the transformed curves.
///
/// ## Parameters
/// * `market_store` - Market store to apply the transformations.
/// * `transformations` - List of transformations to apply to the market store.
/// 
/// ## Disclaimer
/// All index need to have names, otherwise the function will panic.
///
/// ## Curve Transformations
/// The function applies the following transformations:
///
/// * ParallelShift - Apply a parallel shift to the curve -- shift is derived in forward rates -- forwards rate = fwr(base_curve + shift)
/// * TenorBasedShift - Apply a shift to the curve based using a tenor based term structure -- shift is derived in forward rates -- forwards rate = fwr(base_curve + shift)
/// * BaseAndSpread - Overwrite the curve using a diferent base curve and a spread curve -- spread is not derived in forward rates -- forwards rate = fwr(base_curve) + spread_curve
/// * NewCurveAndSpread - Create a new curve using a base curve and a spread curve -- spread is not derived in forward rates -- forwards rate = fwr(base_curve) + spread_curve
///
pub fn apply_curve_transformations(
    market_store: &MarketStore,
    transformations: Vec<CurveTransformations>,
) -> Result<MarketStore> {
    let mut new_market_store = market_store.clone();

    transformations
        .iter()
        .try_for_each(|transform| -> Result<()> {
            let curve_map = new_market_store.index_store().get_index_map()?;
            match transform.transformation_type {
                TransformationType::ParallelShift => {
                    let mut curves = if transform.apply_to == "All" {
                        curve_map
                            .iter()
                            .map(|(_, v)| {
                                new_market_store
                                    .index_store()
                                    .get_index(*v)
                                    .map(|x| x.clone())
                            })
                            .collect::<std::result::Result<
                                Vec<Arc<RwLock<dyn InterestRateIndexTrait>>>,
                                AtlasError,
                            >>()?
                    } else {
                        // check if the curve exists apply_to curve exists
                        let id = match curve_map.get(&transform.apply_to) {
                            Some(v) => v,
                            None => {
                                return Err(AtlasError::InvalidValueErr(format!(
                                    "Curve {} does not exist",
                                    transform.apply_to
                                ))
                                .into())
                            }
                        };
                        vec![new_market_store.index_store().get_index(*id)?.clone()]
                    };
                    curves.iter_mut().try_for_each(|index| -> Result<()> {
                        let index_name = index.read_index()?.name().unwrap();
                        let shift = match transform.shift_value {
                            Some(v) => v,
                            None => {
                                return Err(AtlasError::MissingRequiredField(
                                    "shift_value".to_string(),
                                )
                                .into())
                            }
                        };
                        let flat = Arc::new(FlatForwardTermStructure::new(
                            new_market_store.reference_date(),
                            shift,
                            transform.rate_definition,
                        ));
                        let composite = Arc::new(CompositeTermStructure::new(
                            flat,
                            index.read_index()?.term_structure()?.clone(),
                        ));

                        let id = curve_map.get(&index_name).unwrap().clone();
                        new_market_store
                            .mut_index_store()
                            .link_term_structure(id, composite)?;
                        Ok(())
                    })?;
                }
                TransformationType::TenorBasedShift => {
                    let mut curves = if transform.apply_to == "All" {
                        curve_map
                            .iter()
                            .map(|(_, v)| {
                                new_market_store
                                    .index_store()
                                    .get_index(*v)
                                    .map(|x| x.clone())
                            })
                            .collect::<std::result::Result<
                                Vec<Arc<RwLock<dyn InterestRateIndexTrait>>>,
                                AtlasError,
                            >>()?
                    } else {
                        // check if the curve exists apply_to curve exists
                        let id = match curve_map.get(&transform.apply_to) {
                            Some(v) => v,
                            None => {
                                return Err(AtlasError::InvalidValueErr(format!(
                                    "Curve {} does not exist",
                                    transform.apply_to
                                ))
                                .into())
                            }
                        };
                        vec![new_market_store.index_store().get_index(*id)?.clone()]
                    };
                    curves.iter_mut().try_for_each(|index| -> Result<()> {
                        let index_name = index.read_index()?.name().unwrap();

                        // check if the shift term structure exists
                        let term_structure = match &transform.shift_term_structure {
                            Some(v) => v,
                            None => {
                                return Err(AtlasError::MissingRequiredField(
                                    "term_structure".to_string(),
                                )
                                .into())
                            }
                        };

                        let reference_date = index
                            .read_index()?
                            .term_structure()?
                            .reference_date()
                            .clone();
                        let (tenors, values): (Vec<Date>, Vec<f64>) = term_structure
                            .iter()
                            .map(|v| (reference_date.clone() + v.tenor.clone(), v.value))
                            .unzip();

                        let shift = Arc::new(ZeroRateTermStructure::new(
                            new_market_store.reference_date(),
                            tenors,
                            values,
                            transform.rate_definition,
                            Interpolator::Linear,
                            true,
                        )?);

                        let composite = Arc::new(CompositeTermStructure::new(
                            shift,
                            index.read_index()?.term_structure()?.clone(),
                        ));

                        let id = curve_map.get(&index_name).unwrap().clone();
                        new_market_store
                            .mut_index_store()
                            .link_term_structure(id, composite)?;

                        return Ok(());
                    })?;
                }
                TransformationType::BaseAndSpread => {
                    // for BaseAndSpread transformation, apply_to cannot be All
                    if transform.apply_to == "All" {
                        return Err(AtlasError::InvalidValueErr(
                            "apply_to cannot be All for BaseAndSpread transformation".to_string(),
                        ));
                    }

                    // check if the curve exists apply_to curve exists
                    let id = match curve_map.get(&transform.apply_to) {
                        Some(v) => v,
                        None => {
                            return Err(AtlasError::InvalidValueErr(format!(
                                "Curve {} does not exist",
                                transform.apply_to
                            ))
                            .into())
                        }
                    };

                    // check if the base curve exists
                    let base = match &transform.base_term_structure {
                        Some(v) => v,
                        None => {
                            return Err(AtlasError::MissingRequiredField(
                                "base_term_structure".to_string(),
                            )
                            .into())
                        }
                    };

                    // check if the base curve exists
                    let id_base = match curve_map.get(base) {
                        Some(v) => v,
                        None => {
                            return Err(AtlasError::InvalidValueErr(format!(
                                "Base curve {} does not exist",
                                base
                            ))
                            .into())
                        }
                    };

                    // get the base term structure
                    let base_term_structure = new_market_store
                        .index_store()
                        .get_index(*id_base)?
                        .read_index()?
                        .term_structure()?;

                    let spread = match &transform.spread_term_structure {
                        Some(v) => v,
                        None => {
                            return Err(AtlasError::MissingRequiredField(
                                "spread_term_structure".to_string(),
                            )
                            .into())
                        }
                    };

                    // create the spread term structure
                    let (tenors, spread_values) =
                        spread.iter().map(|v| (v.tenor.clone(), v.value)).unzip();

                    // create the spread term structure
                    let spread_term_structure = Arc::new(TenorBasedZeroRateTermStructure::new(
                        new_market_store.reference_date(),
                        tenors,
                        spread_values,
                        transform.rate_definition,
                        Interpolator::Linear,
                        true,
                    )?);

                    // create the composite term structure
                    let composite = Arc::new(CompositeTermStructure::new(
                        base_term_structure,
                        spread_term_structure,
                    ));

                    // link the composite term structure to the index
                    new_market_store
                        .mut_index_store()
                        .link_term_structure(*id, composite)?;
                }
                TransformationType::ImplicitBaseAndSpread => {
                    // for BaseAndSpread transformation, apply_to cannot be All
                    if transform.apply_to == "All" {
                        return Err(AtlasError::InvalidValueErr(
                            "apply_to cannot be All for BaseAndSpread transformation".to_string(),
                        ));
                    }

                    // check if the curve exists apply_to curve exists
                    let id = match curve_map.get(&transform.apply_to) {
                        Some(v) => v,
                        None => {
                            return Err(AtlasError::InvalidValueErr(format!(
                                "Curve {} does not exist",
                                transform.apply_to
                            ))
                            .into())
                        }
                    };

                    let index = new_market_store.index_store().get_index(*id)?.clone();
                    let spread_base_structure = index.read_index()?.term_structure()?;

                    // check if the base curve exists
                    let base = match &transform.base_term_structure {
                        Some(v) => v,
                        None => {
                            return Err(AtlasError::MissingRequiredField(
                                "base_term_structure".to_string(),
                            )
                            .into())
                        }
                    };

                    // check if the base curve exists
                    let id_base = match curve_map.get(base) {
                        Some(v) => v,
                        None => {
                            return Err(AtlasError::InvalidValueErr(format!(
                                "Base curve {} does not exist",
                                base
                            ))
                            .into())
                        }
                    };

                    // get the base term structure
                    let base_term_structure = new_market_store
                        .index_store()
                        .get_index(*id_base)?
                        .read_index()?
                        .term_structure()?;

                    let spread_term_structure =  Arc::new(TenorBasedSpreadRateTermStructure::new(
                        spread_base_structure,
                        base_term_structure.clone(),
                    ));

                    // create the composite term structure
                    let composite = Arc::new(CompositeTermStructure::new(
                        base_term_structure,
                        spread_term_structure,
                    ));

                    // link the composite term structure to the index
                    new_market_store
                        .mut_index_store()
                        .link_term_structure(*id, composite)?;


                }
                TransformationType::NewCurveAndSpread => {
                    if transform.apply_to == "All" {
                        return Err(AtlasError::InvalidValueErr(
                            "apply_to cannot be All for BaseAndSpread transformation".to_string(),
                        ));
                    }

                    let base = match &transform.base_term_structure {
                        Some(v) => v,
                        None => {
                            return Err(AtlasError::MissingRequiredField(
                                "base_term_structure".to_string(),
                            )
                            .into())
                        }
                    };

                    let spread = match &transform.spread_term_structure {
                        Some(v) => v,
                        None => {
                            return Err(AtlasError::MissingRequiredField(
                                "spread_term_structure".to_string(),
                            )
                            .into())
                        }
                    };

                    // check if the base curve exists
                    let id_base = match curve_map.get(base) {
                        Some(v) => v,
                        None => {
                            return Err(AtlasError::InvalidValueErr(format!(
                                "Base curve {} does not exist",
                                base
                            ))
                            .into())
                        }
                    };

                    let new_id = new_market_store
                        .mut_index_store()
                        .duplicate_index(*id_base, transform.apply_to.clone())?;

                    let base_term_structure = new_market_store
                        .index_store()
                        .get_index_by_name(transform.apply_to.clone())?
                        .read_index()?
                        .term_structure()?;

                    let (tenors, spread_values) =
                        spread.iter().map(|v| (v.tenor.clone(), v.value)).unzip();

                    let spread_term_structure = Arc::new(TenorBasedZeroRateTermStructure::new(
                        new_market_store.reference_date(),
                        tenors,
                        spread_values,
                        transform.rate_definition,
                        Interpolator::Linear,
                        true,
                    )?);

                    let composite = Arc::new(CompositeTermStructure::new(
                        base_term_structure,
                        spread_term_structure,
                    ));

                    new_market_store
                        .mut_index_store()
                        .link_term_structure(new_id, composite)?;
                }
            }
            Ok(())
        })?;
    Ok(new_market_store)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use crate::{
        core::marketstore::MarketStore, currencies::enums::Currency, math::interpolation::enums::Interpolator, rates::{
            enums::Compounding, indexstore::ReadIndex, interestrate::RateDefinition, interestrateindex::{iborindex::IborIndex, overnightindex::OvernightIndex}, traits::{HasReferenceDate, YieldProvider}, yieldtermstructure::{flatforwardtermstructure::FlatForwardTermStructure, zeroratetermstructure::ZeroRateTermStructure}
        }, time::{
            date::Date,
            enums::{Frequency, TimeUnit},
            period::Period,
        }, utils::{errors::Result, marketstoretransformation::TenorBasedValues}
    };

    use super::{apply_curve_transformations, CurveTransformations, TransformationType};

    pub fn create_store(ref_date: Date)-> Result<MarketStore> {
        let local_currency = Currency::USD;
        let mut market_store = MarketStore::new(ref_date, local_currency);

        let forecast_curve_1 = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            0.02,
            RateDefinition::default(),
        ));

        let forecast_curve_2 = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            0.03,
            RateDefinition::default(),
        ));

        let discount_curve = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            0.05,
            RateDefinition::default(),
        ));
        
        let zero_rate = Arc::new(ZeroRateTermStructure::new(
            ref_date,
            vec![ref_date, ref_date + Period::new(1, TimeUnit::Years), ref_date + Period::new(2, TimeUnit::Years), ref_date + Period::new(3, TimeUnit::Years) ],
            vec![0.1, 0.03, 0.04, 0.05],
            RateDefinition::default(),
            Interpolator::Linear,
            true,
        )?);

        let zero_rate_base = Arc::new(ZeroRateTermStructure::new(
            ref_date,
            vec![ref_date, ref_date + Period::new(1, TimeUnit::Years), ref_date + Period::new(2, TimeUnit::Years), ref_date + Period::new(3, TimeUnit::Years) ],
            vec![0.0, 0.02, 0.03, 0.04],
            RateDefinition::default(),
            Interpolator::Linear,
            true,
        )?);

        let mut ibor_fixings = HashMap::new();
        ibor_fixings.insert(Date::new(2021, 9, 1), 0.02); // today
        ibor_fixings.insert(Date::new(2021, 8, 31), 0.02); // yesterday

        let ibor_index = IborIndex::new(forecast_curve_1.reference_date())
            .with_fixings(ibor_fixings)
            .with_term_structure(forecast_curve_1)
            .with_frequency(Frequency::Annual)
            .with_name(Some("ICP".to_string()));

        market_store
            .mut_index_store()
            .add_index(0, Arc::new(RwLock::new(ibor_index)))?;

        let mut overnight_fixings = HashMap::new();
        overnight_fixings.insert(Date::new(2020, 9, 1), 0.06); // today
        overnight_fixings.insert(Date::new(2020, 8, 31), 0.06); // yesterday

        let overnigth_index = OvernightIndex::new(forecast_curve_2.reference_date())
            .with_term_structure(forecast_curve_2)
            .with_fixings(overnight_fixings)
            .with_name(Some("overnight_index".to_string()));

        market_store
            .mut_index_store()
            .add_index(1, Arc::new(RwLock::new(overnigth_index)))?;

        let discount_index = IborIndex::new(discount_curve.reference_date())
            .with_term_structure(discount_curve)
            .with_frequency(Frequency::Annual)
            .with_name(Some("discount_index".to_string()));

        market_store
            .mut_index_store()
            .add_index(2, Arc::new(RwLock::new(discount_index)))?;

        let zero_index = IborIndex::new(zero_rate.reference_date())
            .with_term_structure(zero_rate)
            .with_frequency(Frequency::Annual)
            .with_name(Some("zero_index".to_string()));

        market_store
            .mut_index_store()
            .add_index(3, Arc::new(RwLock::new(zero_index)))?;

        let zero_index_spread = IborIndex::new(zero_rate_base.reference_date())
            .with_term_structure(zero_rate_base)
            .with_frequency(Frequency::Annual)
            .with_name(Some("zero_index_base".to_string()));

        market_store
            .mut_index_store()
            .add_index(4, Arc::new(RwLock::new(zero_index_spread)))?;

        return Ok(market_store);
    }

    #[test]
    fn test_marketstoretransformation() -> Result<()> {
        let date = Date::new(2024, 3, 21);
        let market_store = create_store(date)?;

        let index = market_store
            .index_store()
            .get_index_by_name("ICP".to_string())
            .unwrap();

        let term_structure = index.read_index().unwrap().term_structure().unwrap();

        let fwd_start = market_store.reference_date();
        let fwd_end = market_store.reference_date() + Period::new(1, TimeUnit::Years);

        let rate = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        let transformations = vec![CurveTransformations {
            apply_to: "All".to_string(),
            transformation_type: TransformationType::ParallelShift,
            shift_value: Some(0.01),
            rate_definition: RateDefinition::default(),
            shift_term_structure: None,
            base_term_structure: None,
            spread_term_structure: None,
        }];

        let new_market_store = apply_curve_transformations(&market_store, transformations).unwrap();

        let index = new_market_store
            .index_store()
            .get_index_by_name("ICP".to_string())
            .unwrap();

        let term_structure = index.read_index().unwrap().term_structure().unwrap();
        let second_rate = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        println!("Rate: {:?}", rate);
        println!("Second Rate: {:?}", second_rate);

        assert!((rate + second_rate - 0.05).abs() < 0.00001);

        let first_curve_map = market_store.index_store().get_index_map().unwrap();
        let second_curve_map = new_market_store.index_store().get_index_map().unwrap();

        for (k, v) in first_curve_map.iter() {
            let second_v = second_curve_map.get(k).unwrap();
            assert_eq!(v, second_v);
        }
        assert_eq!(first_curve_map.len(), second_curve_map.len());

        Ok(())
    }
    #[test]
    fn test_apply_curve_transformations_to_one() -> Result<()> {
        let date = Date::new(2024, 3, 21);
        let market_store = create_store(date).unwrap();

        let transformations = vec![CurveTransformations {
            apply_to: "ICP".to_string(),
            transformation_type: TransformationType::ParallelShift,
            shift_value: Some(0.01),
            rate_definition: RateDefinition::default(),
            shift_term_structure: None,
            base_term_structure: None,
            spread_term_structure: None,
        }];

        let index = market_store
            .index_store()
            .get_index_by_name("ICP".to_string())
            .unwrap();
        let term_structure = index.read_index().unwrap().term_structure().unwrap();

        let fwd_start = market_store.reference_date();
        let fwd_end = market_store.reference_date() + Period::new(1, TimeUnit::Years);
        let rate = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        let new_market_store = apply_curve_transformations(&market_store, transformations).unwrap();

        let index = new_market_store
            .index_store()
            .get_index_by_name("ICP".to_string())
            .unwrap();
        let term_structure = index.read_index().unwrap().term_structure().unwrap();
        let second_rate = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        println!("Rate: {:?}", rate);
        println!("Second Rate: {:?}", second_rate);

        let first_curve_map = market_store.index_store().get_index_map().unwrap();
        let second_curve_map = new_market_store.index_store().get_index_map().unwrap();

        for (k, v) in first_curve_map.iter() {
            let second_v = second_curve_map.get(k).unwrap();
            assert_eq!(v, second_v);
        }

        assert_eq!(first_curve_map.len(), second_curve_map.len());
        assert_ne!(rate, second_rate);

        Ok(())
    }

    #[test]
    fn test_apply_new_curve_and_spread() -> Result<()> {
        let date = Date::new(2024, 3, 21);
        let market_store = create_store(date)?;

        let index = market_store
            .index_store()
            .get_index_by_name("ICP".to_string())
            .unwrap();

        let term_structure = index.read_index().unwrap().term_structure().unwrap();
        let fwd_start = market_store.reference_date() + Period::new(1, TimeUnit::Years);
        let fwd_end = market_store.reference_date() + Period::new(2, TimeUnit::Years);
        let rate_1 = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        println!("Rate 1: {:?}", rate_1);

        let mut spread_term = Vec::new();
        spread_term.push(TenorBasedValues{tenor: Period::new(0, TimeUnit::Years), value: 0.001 as f64});
        spread_term.push(TenorBasedValues{tenor: Period::new(1, TimeUnit::Years), value: 0.01 as f64});
        spread_term.push(TenorBasedValues{tenor: Period::new(2, TimeUnit::Years), value: 0.02 as f64});
        spread_term.push(TenorBasedValues{tenor: Period::new(3, TimeUnit::Years), value: 0.03 as f64});
        spread_term.push(TenorBasedValues{tenor: Period::new(4, TimeUnit::Years), value: 0.04 as f64});
        spread_term.push(TenorBasedValues{tenor: Period::new(5, TimeUnit::Years), value: 0.05 as f64});

        let transformations = vec![CurveTransformations {
            apply_to: "ICP_new".to_string(),
            transformation_type: TransformationType::NewCurveAndSpread,
            shift_value: Some(0.01),
            rate_definition: RateDefinition::default(),
            shift_term_structure: None,
            base_term_structure: Some("ICP".to_string()),
            spread_term_structure: Some(spread_term),
        }];


        let new_market_store = apply_curve_transformations(&market_store, transformations).unwrap();
        
        let index = new_market_store
            .index_store()
            .get_index_by_name("ICP_new".to_string())
            .unwrap();

        let term_structure = index.read_index().unwrap().term_structure().unwrap();
        let fwd_start = market_store.reference_date() + Period::new(1, TimeUnit::Years);
        let fwd_end = market_store.reference_date() + Period::new(2, TimeUnit::Years);
        let rate_2 = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        println!("Rate 2: {:?}", rate_2);

        assert!((rate_1 + 0.01 - rate_2).abs() < 0.0000001);

        Ok(())
    }

    #[test]
    fn test_apply_implict_base_and_spread() -> Result<()> {
        let date = Date::new(2025, 3, 21);
        let market_store = create_store(date)?;

        let index = market_store
            .index_store()
            .get_index_by_name("zero_index".to_string())
            .unwrap();

        let term_structure = index.read_index().unwrap().term_structure().unwrap();
        let fwd_start = market_store.reference_date();
        let fwd_end = market_store.reference_date() + Period::new(1, TimeUnit::Years);
        let rate_1 = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        assert!((rate_1 - 0.03).abs() < 0.0000001);

        let fwd_start = market_store.reference_date() + Period::new(1, TimeUnit::Years);
        let fwd_end = market_store.reference_date() + Period::new(2, TimeUnit::Years);
        let rate_2 = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        assert!((rate_2 - 0.04852405984634041).abs() < 0.0000001);

        let transformations = vec![CurveTransformations {
            apply_to: "zero_index".to_string(),
            transformation_type: TransformationType::ImplicitBaseAndSpread,
            shift_value: None,
            rate_definition: RateDefinition::default(),
            shift_term_structure: None,
            base_term_structure: Some("zero_index_base".to_string()),
            spread_term_structure: None,
        }];

        let new_market_store = apply_curve_transformations(&market_store, transformations).unwrap();
        
        let index = new_market_store
            .index_store()
            .get_index_by_name("zero_index".to_string())
            .unwrap();
        
        let term_structure = index.read_index().unwrap().term_structure().unwrap();
        let fwd_start = market_store.reference_date();
        let fwd_end = market_store.reference_date() + Period::new(1, TimeUnit::Years);
        let rate_1 = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        assert!((rate_1 - 0.03).abs() < 0.0000001);

        let fwd_start = market_store.reference_date() + Period::new(1, TimeUnit::Years);
        let fwd_end = market_store.reference_date() + Period::new(2, TimeUnit::Years);
        let rate_2 = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        assert!((rate_2 - 0.04920500952899519).abs() < 0.0000001);

        Ok(())
    }

    #[test]
    fn test_apply_parallel_shift() -> Result<()> {
        let date = Date::new(2024, 3, 21);
        let market_store = create_store(date)?;

        let fwd_start = market_store.reference_date() + Period::new(1, TimeUnit::Years);
        let fwd_end = market_store.reference_date() + Period::new(2, TimeUnit::Years);

        let index = market_store
            .index_store()
            .get_index_by_name("ICP".to_string())
            .unwrap();
        let term_structure = index.read_index().unwrap().term_structure().unwrap();
        let fwd_rate_1: f64 = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        print!("fwd Rate 1: {:?}", fwd_rate_1);

        let flat_rate_index = FlatForwardTermStructure::new(
            date,
            0.01,
            RateDefinition::default(),
        );

        let fwd_rate_2 = flat_rate_index.forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual).unwrap();
        println!("fwd Rate 2: {:?}", fwd_rate_2);

        let transformations = vec![CurveTransformations {
            apply_to: "ICP".to_string(),
            transformation_type: TransformationType::ParallelShift,
            shift_value: Some(0.01),
            rate_definition: RateDefinition::default(),
            shift_term_structure: None,
            base_term_structure: None,
            spread_term_structure: None,
        }];

        let new_market_store = apply_curve_transformations(&market_store, transformations).unwrap();
        let index = new_market_store
            .index_store()
            .get_index_by_name("ICP".to_string())
            .unwrap();

        let term_structure = index.read_index().unwrap().term_structure().unwrap();
        let fwd_rate_3 = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        println!("fwd Rate 3: {:?}", fwd_rate_3);
        assert!((fwd_rate_1 + fwd_rate_2 - fwd_rate_3).abs() < 0.0000001);

        Ok(())
    }

    #[test]
    fn test_apply_two_transformations() -> Result<()>{
        let date = Date::new(2024, 3, 21);
        let market_store = create_store(date)?;

        let fwd_start = market_store.reference_date() + Period::new(1, TimeUnit::Years);
        let fwd_end = market_store.reference_date() + Period::new(2, TimeUnit::Years);

        let index = market_store
            .index_store()
            .get_index_by_name("ICP".to_string())
            .unwrap();
        let term_structure = index.read_index().unwrap().term_structure().unwrap();
        let fwd_rate_1: f64 = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        print!("fwd Rate 1: {:?}", fwd_rate_1);

        let flat_rate_index = FlatForwardTermStructure::new(
            date,
            0.01,
            RateDefinition::default(),
        );

        let fwd_rate_2 = flat_rate_index.forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual).unwrap();
        println!("fwd Rate 2: {:?}", fwd_rate_2);

        let transformations = vec![CurveTransformations {
            apply_to: "ICP".to_string(),
            transformation_type: TransformationType::ParallelShift,
            shift_value: Some(0.01),
            rate_definition: RateDefinition::default(),
            shift_term_structure: None,
            base_term_structure: None,
            spread_term_structure: None,
            },
            CurveTransformations {
            apply_to: "All".to_string(),
            transformation_type: TransformationType::ParallelShift,
            shift_value: Some(0.01),
            rate_definition: RateDefinition::default(),
            shift_term_structure: None,
            base_term_structure: None,
            spread_term_structure: None,
            }
        ];

        let new_market_store = apply_curve_transformations(&market_store, transformations).unwrap();
        let index = new_market_store
            .index_store()
            .get_index_by_name("ICP".to_string())
            .unwrap();

        let term_structure = index.read_index().unwrap().term_structure().unwrap();
        let fwd_rate_3 = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        println!("fwd Rate 3: {:?}", fwd_rate_3);
        assert!((fwd_rate_1 + 2.0*fwd_rate_2 - fwd_rate_3).abs() < 0.0000001);

        Ok(())
    }

    #[test]
    fn test_apply_new_curve_and_spred_and_then_parallel_shift() -> Result<()> {	
        let date = Date::new(2021, 3, 21);
        let market_store = create_store(date)?;

        let index = market_store
            .index_store()
            .get_index_by_name("ICP".to_string())
            .unwrap();

        let term_structure = index.read_index().unwrap().term_structure().unwrap();
        let fwd_start = market_store.reference_date() + Period::new(1, TimeUnit::Years);
        let fwd_end = market_store.reference_date() + Period::new(2, TimeUnit::Years);
        let rate_1 = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        println!("Rate 1: {:?}", rate_1);

        let mut spread_term = Vec::new();
        spread_term.push(TenorBasedValues{tenor: Period::new(0, TimeUnit::Years), value: 0.001 as f64});
        spread_term.push(TenorBasedValues{tenor: Period::new(1, TimeUnit::Years), value: 0.01 as f64});
        spread_term.push(TenorBasedValues{tenor: Period::new(2, TimeUnit::Years), value: 0.02 as f64});
        spread_term.push(TenorBasedValues{tenor: Period::new(3, TimeUnit::Years), value: 0.03 as f64});
        spread_term.push(TenorBasedValues{tenor: Period::new(4, TimeUnit::Years), value: 0.04 as f64});
        spread_term.push(TenorBasedValues{tenor: Period::new(5, TimeUnit::Years), value: 0.05 as f64});

        let transformations = vec![CurveTransformations {
                apply_to: "ICP_new".to_string(),
                transformation_type: TransformationType::NewCurveAndSpread,
                shift_value: Some(0.01),
                rate_definition: RateDefinition::default(),
                shift_term_structure: None,
                base_term_structure: Some("ICP".to_string()),
                spread_term_structure: Some(spread_term),
            },
                CurveTransformations {
                apply_to: "All".to_string(),
                transformation_type: TransformationType::ParallelShift,
                shift_value: Some(0.02),
                rate_definition: RateDefinition::default(),
                shift_term_structure: None,
                base_term_structure: None,
                spread_term_structure: None,
            }        
        ];

        let new_market_store = apply_curve_transformations(&market_store, transformations).unwrap();
        
        let index = new_market_store
            .index_store()
            .get_index_by_name("ICP_new".to_string())
            .unwrap();

        let term_structure = index.read_index().unwrap().term_structure().unwrap();
        let fwd_start = market_store.reference_date() + Period::new(1, TimeUnit::Years);
        let fwd_end = market_store.reference_date() + Period::new(2, TimeUnit::Years);
        let rate_2 = term_structure
            .forward_rate(fwd_start, fwd_end, Compounding::Simple, Frequency::Annual)
            .unwrap();

        println!("Rate 2: {:?}", rate_2);
        assert!((rate_1 + 0.02960250476 - rate_2).abs() < 0.0000001);

        Ok(())
    }

}
