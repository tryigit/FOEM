/// Embeds the LICENSE file contents into the compiled binary.
pub const LICENSE_TEXT: &str = include_str!("../LICENSE");

pub const CRYPTO_DONATIONS: &[(&str, f32, bool, f32)] = &[
    ("USDT (TRC20): TQGTsbqawRHhv35UMxjHo14mieUGWXyQzk", 11.0, true, 0.0),
    ("XMR: 85m61iuWiwp24g8NRXoMKdW25ayVWFzYf5BoAqvgGpLACLuMsXbzGbWR9mC8asnCSfcyHN3dZgEX8KZh2pTc9AzWGXtrEUv", 10.0, true, 0.0),
    ("USDT/USDC (ERC20/BEP20): 0x1a4b9e55e268e6969492a70515a5fd9fd4e6ea8b", 11.0, true, 6.0),
    ("Binance User ID: 114574830", 11.0, false, 0.0),
];

pub const FIAT_DONATIONS: &[(&str, &str)] = &[
    ("PayPal", "https://www.paypal.me/tryigitx"),
    ("Buy Me a Coffee", "https://buymeacoffee.com/yigitx"),
];

pub const COMMUNITY_LINKS: &[(&str, &str)] = &[
    ("GitHub", "https://github.com/tryigit/FOEM"),
    ("Telegram Channel", "https://t.me/cleverestech"),
    ("Report Issue", "https://github.com/tryigit/FOEM/issues"),
];
