// I dont like repeating.
macro_rules! bidirectional_str_enum {
    ($enum_name:ident, {
        $($property:ident => $code:expr => $emoji:expr),* $(,)?
    }) => {
        #[derive(Copy, Clone)]
        pub enum $enum_name {
            $($property),*
        }

        impl $enum_name {
            pub const FLAGS: [Flags; 255] = [$(Self::$property),*];

            pub fn to_code(self) -> &'static str {
                match self {
                    $(Self::$property => $code),*
                }
            }

            pub fn from_code(code: &str) -> Option<$enum_name> {
                match code {
                    $($code => Some(Self::$property)),*,
                    _ => None
                }
            }

            pub fn to_emoji(self) -> &'static str {
                match self {
                    $(Self::$property => $emoji),*
                }
            }

            #[allow(unused)]
            pub fn from_emoji(emoji: &str) -> Option<$enum_name> {
                match emoji {
                    $($emoji => Some(Self::$property)),*,
                    _ => None
                }
            }
        }
    };
}

bidirectional_str_enum!(Flags, {
    Ad => "ad" => "🇦🇩",
    Ae => "ae" => "🇦🇪",
    Af => "af" => "🇦🇫",
    Ag => "ag" => "🇦🇬",
    Ai => "ai" => "🇦🇮",
    Al => "al" => "🇦🇱",
    Am => "am" => "🇦🇲",
    Ao => "ao" => "🇦🇴",
    Aq => "aq" => "🇦🇶",
    Ar => "ar" => "🇦🇷",
    As => "as" => "🇦🇸",
    At => "at" => "🇦🇹",
    Au => "au" => "🇦🇺",
    Aw => "aw" => "🇦🇼",
    Ax => "ax" => "🇦🇽",
    Az => "az" => "🇦🇿",
    Ba => "ba" => "🇧🇦",
    Bb => "bb" => "🇧🇧",
    Bd => "bd" => "🇧🇩",
    Be => "be" => "🇧🇪",
    Bf => "bf" => "🇧🇫",
    Bg => "bg" => "🇧🇬",
    Bh => "bh" => "🇧🇭",
    Bi => "bi" => "🇧🇮",
    Bj => "bj" => "🇧🇯",
    Bl => "bl" => "🇧🇱",
    Bm => "bm" => "🇧🇲",
    Bn => "bn" => "🇧🇳",
    Bo => "bo" => "🇧🇴",
    Bq => "bq" => "🇧🇶",
    Br => "br" => "🇧🇷",
    Bs => "bs" => "🇧🇸",
    Bt => "bt" => "🇧🇹",
    Bv => "bv" => "🇧🇻",
    Bw => "bw" => "🇧🇼",
    By => "by" => "🇧🇾",
    Bz => "bz" => "🇧🇿",
    Ca => "ca" => "🇨🇦",
    Cc => "cc" => "🇨🇨",
    Cd => "cd" => "🇨🇩",
    Cf => "cf" => "🇨🇫",
    Cg => "cg" => "🇨🇬",
    Ch => "ch" => "🇨🇭",
    Ci => "ci" => "🇨🇮",
    Ck => "ck" => "🇨🇰",
    Cl => "cl" => "🇨🇱",
    Cm => "cm" => "🇨🇲",
    Cn => "cn" => "🇨🇳",
    Co => "co" => "🇨🇴",
    Cr => "cr" => "🇨🇷",
    Cu => "cu" => "🇨🇺",
    Cv => "cv" => "🇨🇻",
    Cw => "cw" => "🇨🇼",
    Cx => "cx" => "🇨🇽",
    Cy => "cy" => "🇨🇾",
    Cz => "cz" => "🇨🇿",
    De => "de" => "🇩🇪",
    Dj => "dj" => "🇩🇯",
    Dk => "dk" => "🇩🇰",
    Dm => "dm" => "🇩🇲",
    Do => "do" => "🇩🇴",
    Dz => "dz" => "🇩🇿",
    Ec => "ec" => "🇪🇨",
    Ee => "ee" => "🇪🇪",
    Eg => "eg" => "🇪🇬",
    Eh => "eh" => "🇪🇭",
    Er => "er" => "🇪🇷",
    Es => "es" => "🇪🇸",
    Et => "et" => "🇪🇹",
    Eu => "eu" => "🇪🇺",
    Fi => "fi" => "🇫🇮",
    Fj => "fj" => "🇫🇯",
    Fk => "fk" => "🇫🇰",
    Fm => "fm" => "🇫🇲",
    Fo => "fo" => "🇫🇴",
    Fr => "fr" => "🇫🇷",
    Ga => "ga" => "🇬🇦",
    Gb => "gb" => "🇬🇧",
    Gd => "gd" => "🇬🇩",
    Ge => "ge" => "🇬🇪",
    Gf => "gf" => "🇬🇫",
    Gg => "gg" => "🇬🇬",
    Gh => "gh" => "🇬🇭",
    Gi => "gi" => "🇬🇮",
    Gl => "gl" => "🇬🇱",
    Gm => "gm" => "🇬🇲",
    Gn => "gn" => "🇬🇳",
    Gp => "gp" => "🇬🇵",
    Gq => "gq" => "🇬🇶",
    Gr => "gr" => "🇬🇷",
    Gs => "gs" => "🇬🇸",
    Gt => "gt" => "🇬🇹",
    Gu => "gu" => "🇬🇺",
    Gw => "gw" => "🇬🇼",
    Gy => "gy" => "🇬🇾",
    Hk => "hk" => "🇭🇰",
    Hm => "hm" => "🇭🇲",
    Hn => "hn" => "🇭🇳",
    Hr => "hr" => "🇭🇷",
    Ht => "ht" => "🇭🇹",
    Hu => "hu" => "🇭🇺",
    Id => "id" => "🇮🇩",
    Ie => "ie" => "🇮🇪",
    Il => "il" => "🇮🇱",
    Im => "im" => "🇮🇲",
    In => "in" => "🇮🇳",
    Io => "io" => "🇮🇴",
    Iq => "iq" => "🇮🇶",
    Ir => "ir" => "🇮🇷",
    Is => "is" => "🇮🇸",
    It => "it" => "🇮🇹",
    Je => "je" => "🇯🇪",
    Jm => "jm" => "🇯🇲",
    Jo => "jo" => "🇯🇴",
    Jp => "jp" => "🇯🇵",
    Ke => "ke" => "🇰🇪",
    Kg => "kg" => "🇰🇬",
    Kh => "kh" => "🇰🇭",
    Ki => "ki" => "🇰🇮",
    Km => "km" => "🇰🇲",
    Kn => "kn" => "🇰🇳",
    Kp => "kp" => "🇰🇵",
    Kr => "kr" => "🇰🇷",
    Kw => "kw" => "🇰🇼",
    Ky => "ky" => "🇰🇾",
    Kz => "kz" => "🇰🇿",
    La => "la" => "🇱🇦",
    Lb => "lb" => "🇱🇧",
    Lc => "lc" => "🇱🇨",
    Li => "li" => "🇱🇮",
    Lk => "lk" => "🇱🇰",
    Lr => "lr" => "🇱🇷",
    Ls => "ls" => "🇱🇸",
    Lt => "lt" => "🇱🇹",
    Lu => "lu" => "🇱🇺",
    Lv => "lv" => "🇱🇻",
    Ly => "ly" => "🇱🇾",
    Ma => "ma" => "🇲🇦",
    Mc => "mc" => "🇲🇨",
    Md => "md" => "🇲🇩",
    Me => "me" => "🇲🇪",
    Mf => "mf" => "🇲🇫",
    Mg => "mg" => "🇲🇬",
    Mh => "mh" => "🇲🇭",
    Mk => "mk" => "🇲🇰",
    Ml => "ml" => "🇲🇱",
    Mm => "mm" => "🇲🇲",
    Mn => "mn" => "🇲🇳",
    Mo => "mo" => "🇲🇴",
    Mp => "mp" => "🇲🇵",
    Mq => "mq" => "🇲🇶",
    Mr => "mr" => "🇲🇷",
    Ms => "ms" => "🇲🇸",
    Mt => "mt" => "🇲🇹",
    Mu => "mu" => "🇲🇺",
    Mv => "mv" => "🇲🇻",
    Mw => "mw" => "🇲🇼",
    Mx => "mx" => "🇲🇽",
    My => "my" => "🇲🇾",
    Mz => "mz" => "🇲🇿",
    Na => "na" => "🇳🇦",
    Nc => "nc" => "🇳🇨",
    Ne => "ne" => "🇳🇪",
    Nf => "nf" => "🇳🇫",
    Ng => "ng" => "🇳🇬",
    Ni => "ni" => "🇳🇮",
    Nl => "nl" => "🇳🇱",
    No => "no" => "🇳🇴",
    Np => "np" => "🇳🇵",
    Nr => "nr" => "🇳🇷",
    Nu => "nu" => "🇳🇺",
    Nz => "nz" => "🇳🇿",
    Om => "om" => "🇴🇲",
    Pa => "pa" => "🇵🇦",
    Pe => "pe" => "🇵🇪",
    Pf => "pf" => "🇵🇫",
    Pg => "pg" => "🇵🇬",
    Ph => "ph" => "🇵🇭",
    Pk => "pk" => "🇵🇰",
    Pl => "pl" => "🇵🇱",
    Pm => "pm" => "🇵🇲",
    Pn => "pn" => "🇵🇳",
    Pr => "pr" => "🇵🇷",
    Ps => "ps" => "🇵🇸",
    Pt => "pt" => "🇵🇹",
    Pw => "pw" => "🇵🇼",
    Py => "py" => "🇵🇾",
    Qa => "qa" => "🇶🇦",
    Re => "re" => "🇷🇪",
    Ro => "ro" => "🇷🇴",
    Rs => "rs" => "🇷🇸",
    Ru => "ru" => "🇷🇺",
    Rw => "rw" => "🇷🇼",
    Sa => "sa" => "🇸🇦",
    Sb => "sb" => "🇸🇧",
    Sc => "sc" => "🇸🇨",
    Sd => "sd" => "🇸🇩",
    Se => "se" => "🇸🇪",
    Sg => "sg" => "🇸🇬",
    Sh => "sh" => "🇸🇭",
    Si => "si" => "🇸🇮",
    Sj => "sj" => "🇸🇯",
    Sk => "sk" => "🇸🇰",
    Sl => "sl" => "🇸🇱",
    Sm => "sm" => "🇸🇲",
    Sn => "sn" => "🇸🇳",
    So => "so" => "🇸🇴",
    Sr => "sr" => "🇸🇷",
    Ss => "ss" => "🇸🇸",
    St => "st" => "🇸🇹",
    Sv => "sv" => "🇸🇻",
    Sx => "sx" => "🇸🇽",
    Sy => "sy" => "🇸🇾",
    Sz => "sz" => "🇸🇿",
    Tc => "tc" => "🇹🇨",
    Td => "td" => "🇹🇩",
    Tf => "tf" => "🇹🇫",
    Tg => "tg" => "🇹🇬",
    Th => "th" => "🇹🇭",
    Tj => "tj" => "🇹🇯",
    Tk => "tk" => "🇹🇰",
    Tl => "tl" => "🇹🇱",
    Tm => "tm" => "🇹🇲",
    Tn => "tn" => "🇹🇳",
    To => "to" => "🇹🇴",
    Tr => "tr" => "🇹🇷",
    Tt => "tt" => "🇹🇹",
    Tv => "tv" => "🇹🇻",
    Tw => "tw" => "🇹🇼",
    Tz => "tz" => "🇹🇿",
    Uk => "uk" => "🇺🇦",
    Ug => "ug" => "🇺🇬",
    Um => "um" => "🇺🇲",
    Us => "us" => "🇺🇸",
    Uy => "uy" => "🇺🇾",
    Uz => "uz" => "🇺🇿",
    Va => "va" => "🇻🇦",
    Vc => "vc" => "🇻🇨",
    Ve => "ve" => "🇻🇪",
    Vg => "vg" => "🇻🇬",
    Vi => "vi" => "🇻🇮",
    Vn => "vn" => "🇻🇳",
    Vu => "vu" => "🇻🇺",
    Wf => "wf" => "🇼🇫",
    Ws => "ws" => "🇼🇸",
    Ye => "ye" => "🇾🇪",
    Yt => "yt" => "🇾🇹",
    Za => "za" => "🇿🇦",
    Zm => "zm" => "🇿🇲",
    Zw => "zw" => "🇿🇼",
    GayPrideFlag => "gay_pride_flag" => "🏳️‍🌈",
    TransgenderFlag => "transgender_flag" => "🏳️‍⚧️",
    CheckeredFlag => "checkered_flag" => "🏁",
    PirateFlag => "pirate_flag" => "🏴‍☠️",
    UnitedNations => "united_nations" => "🇺🇳",
});
