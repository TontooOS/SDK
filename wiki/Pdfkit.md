# PDFKit

PDFKit is a sandboxed PDF rendering and editing framework for TontooOS.
It provides a full editor (annotations, forms, page operations), a
sandboxed render helper binary, and TontooUI integration elements
(`PdfPreview`, `PdfPageControl`, `PdfAnnotationBar`, `PdfEditorView`).

## Usage

### 1. Add the feature

```toml
[dependencies]
sdk = { path = "/Library/System/sdk", features = ["PDFKit"] }
```

### 2. Crate root

```rust
sdk::preinclude!();
```

### 3. Simple example

```rust
use PDFKit::{Document, PdfEditor, AnnotationColor};

fn main() {
    let doc = Document::open("/path/to/file.pdf").unwrap();
    println!("Pages: {}", doc.page_count());

    let report = doc.security_report();
    if report.has_javascript {
        println!("Warning: JS in PDF!");
    }

    let mut editor = PdfEditor::new(doc);
    editor.add_highlight(1, vec![[72.0, 700.0, 200.0, 716.0]], AnnotationColor::YELLOW).unwrap();
    editor.add_note(1, [72.0, 640.0, 92.0, 660.0], "Review").unwrap();
    editor.save_to(std::path::Path::new("/tmp/out.pdf")).unwrap();
}
```

### 4. TontooUI integration

```rust
use TontooUI::prelude::*;
use PDFKit::ui_elements::{PdfEditorView, PdfPreview, PdfPageControl};

fn build_ui() -> View {
    let editor = PdfEditorView::new("/tmp/doc.pdf")
        .page(1)
        .scale(1.5)
        .on_tool(|tool| println!("Tool: {tool:?}"))
        .on_page_change(|p| println!("Page: {p}"));

    let preview = PdfPreview::new("/tmp/doc.pdf").page(1).dark(true);
    let nav = PdfPageControl::new(10).current(1).on_change(|p| println!("Page {p}"));

    View::new(editor).with_frame(0.0, 0.0, 800.0, 600.0)
}
```

### 5. Direct rendering

```rust
use PDFKit::RenderRequest;

let req = RenderRequest::new("/tmp/doc.pdf", 1, 1.5, false).unwrap();
let (png, receipt) = req.render().unwrap();
// png = page PNG as bytes
```

## Key API

| Type / Function | Purpose |
|---|---|
| `Document::open(path)` | Open and validate a PDF |
| `Document::page_count()` | Number of pages |
| `Document::security_report()` | Active content scan (JS, launch, embedded) |
| `PdfEditor` | Annotations, forms, page ops, save |
| `PdfPreview::new(path)` | TontooUI single-page renderer (sandboxed) |
| `PdfPageControl::new(total)` | TontooUI prev/next page bar |
| `PdfEditorView::new(path)` | Full editor layout element |
| `RenderRequest::new(path, page, scale, dark)` | Direct page render to PNG |

## Important

- PDFs are rendered **sandboxed** (own process, 20s timeout)
- Active content (JS, Launch) is **never** executed
- SF Pro Font and theme colors (`#1d1d1d` dark / `#ececec` light) are automatic

## Cross References

- [MAIN.md](MAIN.md) -- SDK entry point
- [Sdk.md](Sdk.md) -- system bindings and framework modules
