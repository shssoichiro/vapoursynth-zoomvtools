use vapoursynth::{export_vapoursynth_plugin, plugins::Metadata};

pub const PLUGIN_IDENTIFIER: &str = "com.soichiro.zoomvtools";
pub const PLUGIN_NAME: &str = "zoomvtools";

export_vapoursynth_plugin! {
    Metadata {
        identifier: PLUGIN_IDENTIFIER,
        namespace: "zoomv",
        name: "ZooMVTools",
        read_only: true,
    },
    [
        // TODO: Add functions
    ]
}
