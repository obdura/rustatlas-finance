use crate::{models::traits::Model, visitors::{fixingvisitor::fixingvisitor::FixingVisitor, npvvisitors::npvconstvisitor::NPVConstVisitor}};


/// # ParValue
/// ParValue is a cost function that calculates the NPV of a generic instrument.
///
/// ## Parameters
/// * `eval` - The instrument to evaluate
/// * `market_data` - The market data to use for evaluation
pub struct ParValue<'a, T> {
   pub eval: &'a T,
   pub npv_visitor: Box<NPVConstVisitor<'a>>,
   pub fixing_visitor: Box<FixingVisitor<'a>>,
   pub target_cost: Option<f64>,
}

impl<'a, T> ParValue<'a, T> {
    pub fn new(eval: &'a T, model: &'a dyn Model) -> Self {
        let npv_visitor = NPVConstVisitor::new(model, true);
        let fixing_visitor = FixingVisitor::new(model);
        ParValue {
            eval,
            npv_visitor: Box::new(npv_visitor),
            fixing_visitor: Box::new(fixing_visitor),
            target_cost: None,
        }
    }

    // create a new ParValue with NVPConstVisitor in local currency
    pub fn new_with_local_currency_npv(eval: &'a T, model: &'a dyn Model) -> Self {
        let mut npv_visitor = NPVConstVisitor::new(model, true);
        npv_visitor.set_in_local_currency(true);
        let fixing_visitor = FixingVisitor::new(model);
        ParValue {
            eval,
            npv_visitor: Box::new(npv_visitor),
            fixing_visitor: Box::new(fixing_visitor),
            target_cost: None,
        }
    }

    pub fn set_target_cost(&mut self, target_cost: f64) -> &mut Self {
        self.target_cost = Some(target_cost);
        self
    } 

}


/// # ParValueConstVisitor
/// ParValueConstVisitor is a visitor that calculates the par rate/spread of a generic instrument.
/// 
/// ## Parameters
/// * `market_data` - The market data to use for evaluation
/// * `target_cost` - The target cost to use for evaluation is optional
/// 
/// ## Notes
/// * for swaps instruments, the target cost is the npv of the first leg and the par rate is calculated for the second leg
///    
pub struct ParValueConstVisitor<'a> {
    pub model: &'a dyn Model,
    pub target_cost: Option<f64>,
}

impl<'a> ParValueConstVisitor<'a> {
    pub fn new(model: &'a dyn Model) -> Self {
        ParValueConstVisitor { 
            model: model,
            target_cost: None,
        }
    }

    pub fn new_with_target_cost(model: &'a dyn Model, target_cost: f64) -> Self {
        ParValueConstVisitor { 
            model: model,
            target_cost: Some(target_cost),
        }
    }

    pub fn set_target_cost(&mut self, target_cost: f64) -> &mut Self {
        self.target_cost = Some(target_cost);
        self
    }   
}