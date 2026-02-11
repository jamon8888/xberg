//! Defines the [PdfiumApiVersion] enum, the set of Pdfium API versions supported by `pdfium-render`.

/// A specific Pdfium `FPDF_*` API release version.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PdfiumApiVersion {
    Future, // For changes published to Pdfium's repository but yet to be released in a binary
    V7543,
    V7350,
    V7215,
    V7123,
    V6996,
    V6721,
    V6666,
    V6611,
    V6569,
    V6555,
    V6490,
    V6406,
    V6337,
    V6295,
    V6259,
    V6164,
    V6124,
    V6110,
    V6084,
    V6043,
    V6015,
    V5961,
}

impl PdfiumApiVersion {
    /// Returns the currently selected `PdfiumApiVersion` based on compile-time feature flags.
    /// When multiple version features are enabled (which is common since Cargo features are additive),
    /// the highest enabled version is returned.
    pub(crate) fn current() -> Self {
        #[cfg(feature = "pdfium_future")]
        return PdfiumApiVersion::Future;

        #[cfg(all(feature = "pdfium_7543", not(feature = "pdfium_future")))]
        return PdfiumApiVersion::V7543;

        #[cfg(all(
            feature = "pdfium_7350",
            not(any(feature = "pdfium_future", feature = "pdfium_7543"))
        ))]
        return PdfiumApiVersion::V7350;

        #[cfg(all(
            feature = "pdfium_7215",
            not(any(feature = "pdfium_future", feature = "pdfium_7543", feature = "pdfium_7350"))
        ))]
        return PdfiumApiVersion::V7215;

        #[cfg(all(
            feature = "pdfium_7123",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215"
            ))
        ))]
        return PdfiumApiVersion::V7123;

        #[cfg(all(
            feature = "pdfium_6996",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123"
            ))
        ))]
        return PdfiumApiVersion::V6996;

        #[cfg(all(
            feature = "pdfium_6721",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996"
            ))
        ))]
        return PdfiumApiVersion::V6721;

        #[cfg(all(
            feature = "pdfium_6666",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721"
            ))
        ))]
        return PdfiumApiVersion::V6666;

        #[cfg(all(
            feature = "pdfium_6611",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666"
            ))
        ))]
        return PdfiumApiVersion::V6611;

        #[cfg(all(
            feature = "pdfium_6569",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611"
            ))
        ))]
        return PdfiumApiVersion::V6569;

        #[cfg(all(
            feature = "pdfium_6555",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569"
            ))
        ))]
        return PdfiumApiVersion::V6555;

        #[cfg(all(
            feature = "pdfium_6490",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569",
                feature = "pdfium_6555"
            ))
        ))]
        return PdfiumApiVersion::V6490;

        #[cfg(all(
            feature = "pdfium_6406",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569",
                feature = "pdfium_6555",
                feature = "pdfium_6490"
            ))
        ))]
        return PdfiumApiVersion::V6406;

        #[cfg(all(
            feature = "pdfium_6337",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569",
                feature = "pdfium_6555",
                feature = "pdfium_6490",
                feature = "pdfium_6406"
            ))
        ))]
        return PdfiumApiVersion::V6337;

        #[cfg(all(
            feature = "pdfium_6295",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569",
                feature = "pdfium_6555",
                feature = "pdfium_6490",
                feature = "pdfium_6406",
                feature = "pdfium_6337"
            ))
        ))]
        return PdfiumApiVersion::V6295;

        #[cfg(all(
            feature = "pdfium_6259",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569",
                feature = "pdfium_6555",
                feature = "pdfium_6490",
                feature = "pdfium_6406",
                feature = "pdfium_6337",
                feature = "pdfium_6295"
            ))
        ))]
        return PdfiumApiVersion::V6259;

        #[cfg(all(
            feature = "pdfium_6164",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569",
                feature = "pdfium_6555",
                feature = "pdfium_6490",
                feature = "pdfium_6406",
                feature = "pdfium_6337",
                feature = "pdfium_6295",
                feature = "pdfium_6259"
            ))
        ))]
        return PdfiumApiVersion::V6164;

        #[cfg(all(
            feature = "pdfium_6124",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569",
                feature = "pdfium_6555",
                feature = "pdfium_6490",
                feature = "pdfium_6406",
                feature = "pdfium_6337",
                feature = "pdfium_6295",
                feature = "pdfium_6259",
                feature = "pdfium_6164"
            ))
        ))]
        return PdfiumApiVersion::V6124;

        #[cfg(all(
            feature = "pdfium_6110",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569",
                feature = "pdfium_6555",
                feature = "pdfium_6490",
                feature = "pdfium_6406",
                feature = "pdfium_6337",
                feature = "pdfium_6295",
                feature = "pdfium_6259",
                feature = "pdfium_6164",
                feature = "pdfium_6124"
            ))
        ))]
        return PdfiumApiVersion::V6110;

        #[cfg(all(
            feature = "pdfium_6084",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569",
                feature = "pdfium_6555",
                feature = "pdfium_6490",
                feature = "pdfium_6406",
                feature = "pdfium_6337",
                feature = "pdfium_6295",
                feature = "pdfium_6259",
                feature = "pdfium_6164",
                feature = "pdfium_6124",
                feature = "pdfium_6110"
            ))
        ))]
        return PdfiumApiVersion::V6084;

        #[cfg(all(
            feature = "pdfium_6043",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569",
                feature = "pdfium_6555",
                feature = "pdfium_6490",
                feature = "pdfium_6406",
                feature = "pdfium_6337",
                feature = "pdfium_6295",
                feature = "pdfium_6259",
                feature = "pdfium_6164",
                feature = "pdfium_6124",
                feature = "pdfium_6110",
                feature = "pdfium_6084"
            ))
        ))]
        return PdfiumApiVersion::V6043;

        #[cfg(all(
            feature = "pdfium_6015",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569",
                feature = "pdfium_6555",
                feature = "pdfium_6490",
                feature = "pdfium_6406",
                feature = "pdfium_6337",
                feature = "pdfium_6295",
                feature = "pdfium_6259",
                feature = "pdfium_6164",
                feature = "pdfium_6124",
                feature = "pdfium_6110",
                feature = "pdfium_6084",
                feature = "pdfium_6043"
            ))
        ))]
        return PdfiumApiVersion::V6015;

        #[cfg(all(
            feature = "pdfium_5961",
            not(any(
                feature = "pdfium_future",
                feature = "pdfium_7543",
                feature = "pdfium_7350",
                feature = "pdfium_7215",
                feature = "pdfium_7123",
                feature = "pdfium_6996",
                feature = "pdfium_6721",
                feature = "pdfium_6666",
                feature = "pdfium_6611",
                feature = "pdfium_6569",
                feature = "pdfium_6555",
                feature = "pdfium_6490",
                feature = "pdfium_6406",
                feature = "pdfium_6337",
                feature = "pdfium_6295",
                feature = "pdfium_6259",
                feature = "pdfium_6164",
                feature = "pdfium_6124",
                feature = "pdfium_6110",
                feature = "pdfium_6084",
                feature = "pdfium_6043",
                feature = "pdfium_6015"
            ))
        ))]
        return PdfiumApiVersion::V5961;
    }
}
