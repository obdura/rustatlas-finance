use crate::{core::meta::MarketData, visitors::{indexingvisitors::fixingvisitor::FixingVisitor, npvvisitors::npvconstvisitor::NPVConstVisitor}};


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
    pub fn new(eval: &'a T, market_data: &'a [MarketData]) -> Self {
        let npv_visitor = NPVConstVisitor::new(market_data, true);
        let fixing_visitor = FixingVisitor::new(market_data);
        ParValue {
            eval,
            npv_visitor: Box::new(npv_visitor),
            fixing_visitor: Box::new(fixing_visitor),
            target_cost: None,
        }
    }

    // create a new ParValue with NVPConstVisitor in local currency
    pub fn new_with_local_currency_npv(eval: &'a T, market_data: &'a [MarketData]) -> Self {
        let mut npv_visitor = NPVConstVisitor::new(market_data, true);
        npv_visitor.set_in_local_currency(true);
        let fixing_visitor = FixingVisitor::new(market_data);
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
    pub market_data: &'a [MarketData],
    pub target_cost: Option<f64>,
}

impl<'a> ParValueConstVisitor<'a> {
    pub fn new(market_data: &'a [MarketData]) -> Self {
        ParValueConstVisitor { 
            market_data: market_data,
            target_cost: None,
        }
    }

    pub fn new_with_target_cost(market_data: &'a [MarketData], target_cost: f64) -> Self {
        ParValueConstVisitor { 
            market_data: market_data,
            target_cost: Some(target_cost),
        }
    }

    pub fn set_target_cost(&mut self, target_cost: f64) -> &mut Self {
        self.target_cost = Some(target_cost);
        self
    }   
}