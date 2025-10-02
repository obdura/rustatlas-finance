// use std::cell::RefCell;

// use super::super::traits::{HasCashflows, Visit};
// use crate::{
//     core::{meta::MarketRequest, traits::Registrable},
//     utils::errors::Result,
// };

// /// # IndexingVisitor
// /// 
// /// IndexingVisitor is a visitor that registers the cashflows of an instrument
// /// and generates a vector of market requests
// /// 
// /// ## Parameters
// /// * `request` - The vector of market requests
// /// 
// #[deprecated(
//     since = "1.0.0",
//     note = "This struct is deprecated and will be removed in future versions. Please use the new implementation."
// )]
// pub struct IndexingVisitor {
//     request: RefCell<Vec<MarketRequest>>,
// }

// impl IndexingVisitor {
//     pub fn new() -> Self {
//         IndexingVisitor {
//             request: RefCell::new(Vec::new()),
//         }
//     }

//     pub fn request(&self) -> Vec<MarketRequest> {
//         self.request.borrow().clone()
//     }

//     fn visit_cashflows(&self, has_cashflows: &mut dyn HasCashflows) -> Result<()> {
//         let mut requests = self.request.borrow_mut();
//         has_cashflows
//             .mut_cashflows()
//             .try_for_each(|cf| -> Result<()> {
//                 cf.set_id(requests.len());
//                 let request = cf.market_request()?;
//                 requests.push(request);
//                 Ok(())
//             })?;
//         Ok(())
//     }
// }

// impl<T: HasCashflows> Visit<T> for IndexingVisitor {
//     type Output = Result<()>;
//     fn visit(&self, has_cashflows: &mut T) -> Self::Output {
//         self.visit_cashflows(has_cashflows)
//     }
// }

// impl Visit<&mut Box<dyn HasCashflows>> for IndexingVisitor {
//     type Output = Result<()>;
//     fn visit(&self, has_cashflows: &mut &mut Box<dyn HasCashflows>) -> Self::Output {
//         self.visit_cashflows(has_cashflows.as_mut())
//     }
// }





