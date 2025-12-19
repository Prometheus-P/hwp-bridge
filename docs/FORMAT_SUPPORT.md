# Format Support Matrix (WIP)

This file is deliberately blunt. It protects your reputation.

| Feature | Status | Notes |
|---|---:|---|
| OLE container open | ✅ | `hwp_core::HwpOleFile` |
| FileHeader parse | ✅ | encryption/distribution flags |
| BodyText Paragraph text | ✅ | `RecordTag::ParaText` |
| Nested tables | ⚠️ Partial | needs more fixtures |
| Images | ⚠️ Partial | extraction pipeline WIP |
| Header/Footer | ⚠️ Partial | not guaranteed |
| Footnotes/Endnotes | ❌ | planned |
| HWPX (.hwpx) | ❌ | returns `UNSUPPORTED_FORMAT` (blocked by design) |

## Definition of “✅”
✅ = you can run it on a corpus and it doesn’t corrupt output in a way that misleads users.
