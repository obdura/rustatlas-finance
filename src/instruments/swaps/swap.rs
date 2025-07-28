use crate::{cashflows::cashflow::Cashflow, visitors::traits::HasCashflows};

use super::{crosscurrencyswap::CrossCurrencySwap, leg::Leg, vanillairsswap::VanillaIRSSwap};


pub enum Swap{
    VanillaIRSSwap(VanillaIRSSwap),
    CrossCurrencySwap(CrossCurrencySwap),
}

impl HasCashflows for Swap {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_> {
        match self {
            Swap::VanillaIRSSwap(swap) => swap.cashflows(),
            Swap::CrossCurrencySwap(swap) => swap.cashflows(),
        }
    }
    
    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_> {
        match self {
            Swap::VanillaIRSSwap(swap) => swap.mut_cashflows(),
            Swap::CrossCurrencySwap(swap) => swap.mut_cashflows(),
        }
    }
}

impl Swap {
    pub fn id(&self) -> Option<String> {
        match self {
            Swap::VanillaIRSSwap(swap) => swap.id(),
            Swap::CrossCurrencySwap(swap) => swap.id(),
        }
    }

    pub fn first_leg(&self) -> &Leg {
        match self {
            Swap::VanillaIRSSwap(swap) => swap.first_leg(),
            Swap::CrossCurrencySwap(swap) => swap.first_leg(),
        }
    }

    pub fn mut_first_leg(&mut self) -> &mut Leg {
        match self {
            Swap::VanillaIRSSwap(swap) => swap.mut_first_leg(),
            Swap::CrossCurrencySwap(swap) => swap.mut_first_leg(),
        }
    }

    pub fn second_leg(&self) -> &Leg {
        match self {
            Swap::VanillaIRSSwap(swap) => swap.second_leg(),
            Swap::CrossCurrencySwap(swap) => swap.second_leg(),
        }
    }

    pub fn mut_second_leg(&mut self) -> &mut Leg {
        match self {
            Swap::VanillaIRSSwap(swap) => swap.mut_second_leg(),
            Swap::CrossCurrencySwap(swap) => swap.mut_second_leg(),
        }
    }

}


impl std::fmt::Display for Swap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Swap::VanillaIRSSwap(swap) => write!(f, "{}", swap),
            Swap::CrossCurrencySwap(swap) => write!(f, "{}", swap),
        }
    }
}

#[cfg(test)]
mod tests {

    

}
