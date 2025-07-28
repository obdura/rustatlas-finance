use super::traits::CurrencyDetails;

/// # USD
/// Struct for USD currency
pub struct USD;

/// # EUR
/// Struct for EUR currency
pub struct EUR;

/// # JPY
/// Struct for JPY currency
pub struct JPY;

/// # ZAR
/// Struct for ZAR currency
pub struct ZAR;

/// # CLP
/// Struct for CLP currency
pub struct CLP;

/// # CLF
/// Struct for CLF currency
pub struct CLF;

/// # CHF
/// Struct for CHF currency
pub struct CHF;

/// # BRL
/// Struct for BRL currency
pub struct BRL;

/// # COP
/// Struct for COP currency
pub struct COP;

/// # AUD
/// Struct for AUD currency
pub struct AUD;

/// # CAD
/// Struct for CAD currency
pub struct CAD;

/// # CNY
/// Struct for CNY currency
pub struct CNY;

/// # GBP
/// Struct for GBP currency
pub struct GBP;

/// # MXN
/// Struct for MXN currency
pub struct MXN;

/// # NZD
/// Struct for NZD currency
pub struct NZD;

/// # PEN
/// Struct for PEN currency
pub struct PEN;

/// # NOK
/// Struct for NOK currency
pub struct NOK;

/// # SEK
/// Struct for SEK currency
pub struct SEK;

/// # CNH
/// Struct for CNH currency
pub struct CNH;

/// # INR
/// Struct for INR currency
pub struct INR;

/// # TWD
/// Struct for TWD currency
pub struct TWD;

/// # KRW
/// Struct for KRW currency
pub struct KRW;

/// # HKD
/// Struct for HKD currency
pub struct HKD;

/// # DKK
/// Struct for DKK currency
pub struct DKK;

/// # IDR
/// Struct for IDR currency
pub struct IDR;


impl CurrencyDetails for IDR {
    fn code(&self) -> String {
        return "IDR".to_string();
    }
    fn name(&self) -> String {
        return "Indonesian Rupiah".to_string();
    }
    fn symbol(&self) -> String {
        return "Rp".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 360;
    }
}


impl CurrencyDetails for HKD {
    fn code(&self) -> String {
        return "HKD".to_string();
    }
    fn name(&self) -> String {
        return "Hong Kong Dollar".to_string();
    }
    fn symbol(&self) -> String {
        return "HK$".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 344;
    }
}

impl CurrencyDetails for KRW {
    fn code(&self) -> String {
        return "KRW".to_string();
    }
    fn name(&self) -> String {
        return "South Korean Won".to_string();
    }
    fn symbol(&self) -> String {
        return "₩".to_string();
    }
    fn precision(&self) -> u8 {
        return 0;
    }
    fn numeric_code(&self) -> u16 {
        return 410;
    }
}

impl CurrencyDetails for TWD {
    fn code(&self) -> String {
        return "TWD".to_string();
    }
    fn name(&self) -> String {
        return "New Taiwan Dollar".to_string();
    }
    fn symbol(&self) -> String {
        return "NT$".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 901;
    }
}

impl CurrencyDetails for INR {
    fn code(&self) -> String {
        return "INR".to_string();
    }
    fn name(&self) -> String {
        return "Indian Rupee".to_string();
    }
    fn symbol(&self) -> String {
        return "₹".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 356;
    }
}

impl CurrencyDetails for USD {
    fn code(&self) -> String {
        return "USD".to_string();
    }
    fn name(&self) -> String {
        return "US Dollar".to_string();
    }
    fn symbol(&self) -> String {
        return "$".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 840;
    }
}

impl CurrencyDetails for EUR {
    fn code(&self) -> String {
        return "EUR".to_string();
    }
    fn name(&self) -> String {
        return "Euro".to_string();
    }
    fn symbol(&self) -> String {
        return "€".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 978;
    }
}

impl CurrencyDetails for JPY {
    fn code(&self) -> String {
        return "JPY".to_string();
    }
    fn name(&self) -> String {
        return "Japanese Yen".to_string();
    }
    fn symbol(&self) -> String {
        return "¥".to_string();
    }
    fn precision(&self) -> u8 {
        return 0;
    }
    fn numeric_code(&self) -> u16 {
        return 392;
    }
}

impl CurrencyDetails for ZAR {
    fn code(&self) -> String {
        return "ZAR".to_string();
    }
    fn name(&self) -> String {
        return "South African Rand".to_string();
    }
    fn symbol(&self) -> String {
        return "R".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 710;
    }
}

impl CurrencyDetails for CLP {
    fn code(&self) -> String {
        return "CLP".to_string();
    }
    fn name(&self) -> String {
        return "Chilean Peso".to_string();
    }
    fn symbol(&self) -> String {
        return "$".to_string();
    }
    fn precision(&self) -> u8 {
        return 0;
    }
    fn numeric_code(&self) -> u16 {
        return 152;
    }
}

impl CurrencyDetails for CLF {
    fn code(&self) -> String {
        return "CLF".to_string();
    }
    fn name(&self) -> String {
        return "Chilean Unidad de Fomento".to_string();
    }
    fn symbol(&self) -> String {
        return "UF".to_string();
    }
    fn precision(&self) -> u8 {
        return 4;
    }
    fn numeric_code(&self) -> u16 {
        return 990;
    }
}

impl CurrencyDetails for CHF {
    fn code(&self) -> String {
        return "CHF".to_string();
    }
    fn name(&self) -> String {
        return "Swiss Franc".to_string();
    }
    fn symbol(&self) -> String {
        return "Fr".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 756;
    }
}

impl CurrencyDetails for BRL {
    fn code(&self) -> String {
        return "BRL".to_string();
    }
    fn name(&self) -> String {
        return "Brazilian Real".to_string();
    }
    fn symbol(&self) -> String {
        return "R$".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 986;
    }
}

impl CurrencyDetails for COP {
    fn code(&self) -> String {
        return "COP".to_string();
    }
    fn name(&self) -> String {
        return "Colombian Peso".to_string();
    }
    fn symbol(&self) -> String {
        return "$".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 170;
    }
}

impl CurrencyDetails for AUD {
    fn code(&self) -> String {
        return "AUD".to_string();
    }
    fn name(&self) -> String {
        return "Australian Dollar".to_string();
    }
    fn symbol(&self) -> String {
        return "A$".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 36;
    }
}

impl CurrencyDetails for NZD {
    fn code(&self) -> String {
        return "NZD".to_string();
    }
    fn name(&self) -> String {
        return "New Zealand Dollar".to_string();
    }
    fn symbol(&self) -> String {
        return "NZ$".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 554;
    }
}

impl CurrencyDetails for CAD {
    fn code(&self) -> String {
        return "CAD".to_string();
    }
    fn name(&self) -> String {
        return "Canadian Dollar".to_string();
    }
    fn symbol(&self) -> String {
        return "Can$".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 124;
    }
}

impl CurrencyDetails for MXN {
    fn code(&self) -> String {
        return "MXN".to_string();
    }
    fn name(&self) -> String {
        return "Mexican Peso".to_string();
    }
    fn symbol(&self) -> String {
        return "Mex$".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 484;
    }
}

impl CurrencyDetails for PEN {
    fn code(&self) -> String {
        return "PEN".to_string();
    }
    fn name(&self) -> String {
        return "Peruvian Sol".to_string();
    }
    fn symbol(&self) -> String {
        return "S/.".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 604;
    }
}

impl CurrencyDetails for GBP {
    fn code(&self) -> String {
        return "GBP".to_string();
    }
    fn name(&self) -> String {
        return "British Pound".to_string();
    }
    fn symbol(&self) -> String {
        return "£".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 826;
    }
}

impl CurrencyDetails for CNY {
    fn code(&self) -> String {
        return "CNY".to_string();
    }
    fn name(&self) -> String {
        return "Chinese Yuan".to_string();
    }
    fn symbol(&self) -> String {
        return "¥".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 156;
    }
}

impl CurrencyDetails for NOK {
    fn code(&self) -> String {
        return "NOK".to_string();
    }
    fn name(&self) -> String {
        return "Norwegian Krone".to_string();
    }
    fn symbol(&self) -> String {
        return "kr".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 578;
    }
}

impl CurrencyDetails for SEK {
    fn code(&self) -> String {
        return "SEK".to_string();
    }
    fn name(&self) -> String {
        return "Swedish Krona".to_string();
    }
    fn symbol(&self) -> String {
        return "kr".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 752;
    }
}

impl CurrencyDetails for CNH {
    fn code(&self) -> String {
        return "CNH".to_string();
    }
    fn name(&self) -> String {
        return "Chinese Yuan (offshore)".to_string();
    }
    fn symbol(&self) -> String {
        return "¥".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 156;
    }
}

impl CurrencyDetails for DKK {
    fn code(&self) -> String {
        return "DKK".to_string();
    }
    fn name(&self) -> String {
        return "Danish Krone".to_string();
    }
    fn symbol(&self) -> String {
        return "kr".to_string();
    }
    fn precision(&self) -> u8 {
        return 2;
    }
    fn numeric_code(&self) -> u16 {
        return 208;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usd_details() {
        let usd = USD;
        assert_eq!(usd.code(), "USD");
        assert_eq!(usd.name(), "US Dollar");
        assert_eq!(usd.symbol(), "$");
        assert_eq!(usd.precision(), 2);
        assert_eq!(usd.numeric_code(), 840);
    }

    #[test]
    fn test_eur_details() {
        let eur = EUR;
        assert_eq!(eur.code(), "EUR");
        assert_eq!(eur.name(), "Euro");
        assert_eq!(eur.symbol(), "€");
        assert_eq!(eur.precision(), 2);
        assert_eq!(eur.numeric_code(), 978);
    }

    #[test]
    fn test_jpy_details() {
        let jpy = JPY;
        assert_eq!(jpy.code(), "JPY");
        assert_eq!(jpy.name(), "Japanese Yen");
        assert_eq!(jpy.symbol(), "¥");
        assert_eq!(jpy.precision(), 0);
        assert_eq!(jpy.numeric_code(), 392);
    }

    #[test]
    fn test_clp_details() {
        let clp = CLP;
        assert_eq!(clp.code(), "CLP");
        assert_eq!(clp.name(), "Chilean Peso");
        assert_eq!(clp.symbol(), "$");
        assert_eq!(clp.precision(), 0);
        assert_eq!(clp.numeric_code(), 152);
    }

    #[test]
    fn test_cny_details() {
        let cny = CNY;
        assert_eq!(cny.code(), "CNY");
        assert_eq!(cny.name(), "Chinese Yuan");
        assert_eq!(cny.symbol(), "¥");
        assert_eq!(cny.precision(), 2);
        assert_eq!(cny.numeric_code(), 156);
    }

    #[test]
    fn test_inr_details() {
        let inr = INR;
        assert_eq!(inr.code(), "INR");
        assert_eq!(inr.name(), "Indian Rupee");
        assert_eq!(inr.symbol(), "₹");
        assert_eq!(inr.precision(), 2);
        assert_eq!(inr.numeric_code(), 356);
    }

    #[test]
    fn test_nzd_details() {
        let nzd = NZD;
        assert_eq!(nzd.code(), "NZD");
        assert_eq!(nzd.name(), "New Zealand Dollar");
        assert_eq!(nzd.symbol(), "NZ$");
        assert_eq!(nzd.precision(), 2);
        assert_eq!(nzd.numeric_code(), 554);
    }

    #[test]
    fn test_cnh_details() {
        let cnh = CNH;
        assert_eq!(cnh.code(), "CNH");
        assert_eq!(cnh.name(), "Chinese Yuan (offshore)");
        assert_eq!(cnh.symbol(), "¥");
        assert_eq!(cnh.precision(), 2);
        assert_eq!(cnh.numeric_code(), 156);
    }

    #[test]
    fn test_hkd_details() {
        let hkd = HKD;
        assert_eq!(hkd.code(), "HKD");
        assert_eq!(hkd.name(), "Hong Kong Dollar");
        assert_eq!(hkd.symbol(), "HK$");
        assert_eq!(hkd.precision(), 2);
        assert_eq!(hkd.numeric_code(), 344);
    }

    #[test]
    fn test_idr_details() {
        let idr = IDR;
        assert_eq!(idr.code(), "IDR");
        assert_eq!(idr.name(), "Indonesian Rupiah");
        assert_eq!(idr.symbol(), "Rp");
        assert_eq!(idr.precision(), 2);
        assert_eq!(idr.numeric_code(), 360);
    }

    #[test]
    fn test_zar_details() {
        let zar = ZAR;
        assert_eq!(zar.code(), "ZAR");
        assert_eq!(zar.name(), "South African Rand");
        assert_eq!(zar.symbol(), "R");
        assert_eq!(zar.precision(), 2);
        assert_eq!(zar.numeric_code(), 710);
    }

    #[test]
    fn test_clf_details() {
        let clf = CLF;
        assert_eq!(clf.code(), "CLF");
        assert_eq!(clf.name(), "Chilean Unidad de Fomento");
        assert_eq!(clf.symbol(), "UF");
        assert_eq!(clf.precision(), 4);
        assert_eq!(clf.numeric_code(), 990);
    }

    #[test]
    fn test_chf_details() {
        let chf = CHF;
        assert_eq!(chf.code(), "CHF");
        assert_eq!(chf.name(), "Swiss Franc");
        assert_eq!(chf.symbol(), "Fr");
        assert_eq!(chf.precision(), 2);
        assert_eq!(chf.numeric_code(), 756);
    }

    #[test]
    fn test_brl_details() {
        let brl = BRL;
        assert_eq!(brl.code(), "BRL");
        assert_eq!(brl.name(), "Brazilian Real");
        assert_eq!(brl.symbol(), "R$");
        assert_eq!(brl.precision(), 2);
        assert_eq!(brl.numeric_code(), 986);
    }

    #[test]
    fn test_cop_details() {
        let cop = COP;
        assert_eq!(cop.code(), "COP");
        assert_eq!(cop.name(), "Colombian Peso");
        assert_eq!(cop.symbol(), "$");
        assert_eq!(cop.precision(), 2);
        assert_eq!(cop.numeric_code(), 170);
    }

    #[test]
    fn test_aud_details() {
        let aud = AUD;
        assert_eq!(aud.code(), "AUD");
        assert_eq!(aud.name(), "Australian Dollar");
        assert_eq!(aud.symbol(), "A$");
        assert_eq!(aud.precision(), 2);
        assert_eq!(aud.numeric_code(), 36);
    }

    #[test]
    fn test_cad_details() {
        let cad = CAD;
        assert_eq!(cad.code(), "CAD");
        assert_eq!(cad.name(), "Canadian Dollar");
        assert_eq!(cad.symbol(), "Can$");
        assert_eq!(cad.precision(), 2);
        assert_eq!(cad.numeric_code(), 124);
    }

    #[test]
    fn test_mxn_details() {
        let mxn = MXN;
        assert_eq!(mxn.code(), "MXN");
        assert_eq!(mxn.name(), "Mexican Peso");
        assert_eq!(mxn.symbol(), "Mex$");
        assert_eq!(mxn.precision(), 2);
        assert_eq!(mxn.numeric_code(), 484);
    }

    #[test]
    fn test_pen_details() {
        let pen = PEN;
        assert_eq!(pen.code(), "PEN");
        assert_eq!(pen.name(), "Peruvian Sol");
        assert_eq!(pen.symbol(), "S/.");
        assert_eq!(pen.precision(), 2);
        assert_eq!(pen.numeric_code(), 604);
    }

    #[test]
    fn test_gbp_details() {
        let gbp = GBP;
        assert_eq!(gbp.code(), "GBP");
        assert_eq!(gbp.name(), "British Pound");
        assert_eq!(gbp.symbol(), "£");
        assert_eq!(gbp.precision(), 2);
        assert_eq!(gbp.numeric_code(), 826);
    }

    #[test]
    fn test_nok_details() {
        let nok = NOK;
        assert_eq!(nok.code(), "NOK");
        assert_eq!(nok.name(), "Norwegian Krone");
        assert_eq!(nok.symbol(), "kr");
        assert_eq!(nok.precision(), 2);
        assert_eq!(nok.numeric_code(), 578);
    }

    #[test]
    fn test_sek_details() {
        let sek = SEK;
        assert_eq!(sek.code(), "SEK");
        assert_eq!(sek.name(), "Swedish Krona");
        assert_eq!(sek.symbol(), "kr");
        assert_eq!(sek.precision(), 2);
        assert_eq!(sek.numeric_code(), 752);
    }

    #[test]
    fn test_dkk_details() {
        let dkk = DKK;
        assert_eq!(dkk.code(), "DKK");
        assert_eq!(dkk.name(), "Danish Krone");
        assert_eq!(dkk.symbol(), "kr");
        assert_eq!(dkk.precision(), 2);
        assert_eq!(dkk.numeric_code(), 208);
    }
}