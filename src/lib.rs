use anyhow::Error;
use mv_super::Super;
use vapoursynth::{
    api::API,
    core::CoreRef,
    export_vapoursynth_plugin,
    make_filter_function,
    map::Map,
    node::Node,
    plugins::{Filter, FilterArgument, Metadata},
};

mod mv_analyse;
mod mv_compensate;
mod mv_frame;
mod mv_gof;
mod mv_recalculate;
mod mv_super;
mod params;
mod reduce;
#[cfg(test)]
mod tests;
mod util;

pub const PLUGIN_IDENTIFIER: &str = "com.soichiro.zoomvtools";
pub const PLUGIN_NAME: &str = "zoomvtools";

make_filter_function! {
    SuperFunction, "Super"
    #[allow(unused_variables)]
    fn create_super<'core>(
        _api: API,
        _core: CoreRef<'core>,
        clip: Node<'core>,
        hpad: Option<i64>,
        vpad: Option<i64>,
        pel: Option<i64>,
        levels: Option<i64>,
        chroma: Option<i64>,
        sharp: Option<i64>,
        rfilter: Option<i64>,
        pelclip: Option<Node<'core>>,
        opt: Option<i64>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        // `opt` exists for compatibility purposes, but will not be used.
        let mvsuper = Super::new(clip, hpad, vpad, pel, levels, chroma, sharp, rfilter, pelclip)?;

        Ok(Some(Box::new(mvsuper)))
    }
}

export_vapoursynth_plugin! {
    Metadata {
        identifier: PLUGIN_IDENTIFIER,
        namespace: "zoomv",
        name: "ZooMVTools",
        read_only: true,
    },
    [
        SuperFunction::new()
    ]
}
