// Test file for compound word replacements
struct PreviewFormatArg {
    value: String,
}

impl From<PreviewFormatArg> for PreviewFormat {
    fn from(arg: PreviewFormatArg) -> PreviewFormat {
        PreviewFormat::new(arg.value)
    }
}

fn getPreviewFormatOption() -> PreviewFormatOption {
    PreviewFormatOption::default()
}

fn shouldHandlePreviewFormatProperly(preview_format_arg: PreviewFormatArg) {
    let preview_format = PreviewFormat::from(preview_format_arg);
    process_preview_format(preview_format);
}

fn load_preview_format() -> PreviewFormat {
    PreviewFormat::load()
}