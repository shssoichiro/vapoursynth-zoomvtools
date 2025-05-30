use anyhow::Error;
use mv_super::Super;
use vapoursynth::{
    api::API,
    core::CoreRef,
    export_vapoursynth_plugin, make_filter_function,
    map::Map,
    node::Node,
    plugins::{Filter, FilterArgument, Metadata},
};

#[cfg(feature = "bench")]
pub mod average;
#[cfg(feature = "bench")]
pub mod mv_analyse;
#[cfg(feature = "bench")]
pub mod mv_compensate;
#[cfg(feature = "bench")]
pub mod mv_frame;
#[cfg(feature = "bench")]
pub mod mv_gof;
#[cfg(feature = "bench")]
pub mod mv_plane;
#[cfg(feature = "bench")]
pub mod mv_recalculate;
#[cfg(feature = "bench")]
pub mod mv_super;
#[cfg(feature = "bench")]
pub mod pad;
#[cfg(feature = "bench")]
pub mod params;
#[cfg(feature = "bench")]
pub mod reduce;
#[cfg(feature = "bench")]
pub mod refine;
#[cfg(feature = "bench")]
pub mod util;

#[cfg(not(feature = "bench"))]
mod average;
#[cfg(not(feature = "bench"))]
mod mv_analyse;
#[cfg(not(feature = "bench"))]
mod mv_compensate;
#[cfg(not(feature = "bench"))]
mod mv_frame;
#[cfg(not(feature = "bench"))]
mod mv_gof;
#[cfg(not(feature = "bench"))]
mod mv_plane;
#[cfg(not(feature = "bench"))]
mod mv_recalculate;
#[cfg(not(feature = "bench"))]
mod mv_super;
#[cfg(not(feature = "bench"))]
mod pad;
#[cfg(not(feature = "bench"))]
mod params;
#[cfg(not(feature = "bench"))]
mod reduce;
#[cfg(not(feature = "bench"))]
mod refine;
#[cfg(not(feature = "bench"))]
mod util;

#[cfg(test)]
mod tests;

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
