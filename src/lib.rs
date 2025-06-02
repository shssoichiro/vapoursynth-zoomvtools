use anyhow::Error;
use mv_analyse::Analyse;
use mv_super::Super;
use vapoursynth::{
    api::API,
    core::CoreRef,
    export_vapoursynth_plugin, make_filter_function,
    map::Map,
    node::Node,
    plugins::{Filter, FilterArgument, Metadata},
};

#[cfg(test)]
#[macro_use]
mod tests;

#[cfg(feature = "bench")]
pub mod average;
#[cfg(feature = "bench")]
pub mod mv_analyse;
#[cfg(feature = "bench")]
pub mod mv_frame;
#[cfg(feature = "bench")]
pub mod mv_gof;
#[cfg(feature = "bench")]
pub mod mv_plane;
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
mod mv_frame;
#[cfg(not(feature = "bench"))]
mod mv_gof;
#[cfg(not(feature = "bench"))]
mod mv_plane;
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

pub const PLUGIN_IDENTIFIER: &str = "com.soichiro.zoomvtools";
pub const PLUGIN_NAME: &str = "zoomvtools";

make_filter_function! {
    AnalyseFunction, "Analyse"
    #[allow(unused_variables)]
    fn create_analyse<'core>(
        _api: API,
        _core: CoreRef<'core>,
        super_: Node<'core>,
        blksize: Option<i64>,
        blksizev: Option<i64>,
        levels: Option<i64>,
        search: Option<i64>,
        searchparam: Option<i64>,
        pelsearch: Option<i64>,
        isb: Option<i64>,
        lambda: Option<i64>,
        chroma: Option<i64>,
        delta: Option<i64>,
        truemotion: Option<i64>,
        lsad: Option<i64>,
        plevel: Option<i64>,
        global: Option<i64>,
        pnew: Option<i64>,
        pzero: Option<i64>,
        pglobal: Option<i64>,
        overlap: Option<i64>,
        overlapv: Option<i64>,
        divide: Option<i64>,
        badsad: Option<i64>,
        badrange: Option<i64>,
        opt: Option<i64>,
        meander: Option<i64>,
        trymany: Option<i64>,
        fields: Option<i64>,
        tff: Option<i64>,
        search_coarse: Option<i64>,
        dct: Option<i64>,
        clip: Node<'core>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        // `opt` exists for compatibility purposes, but will not be used.
        // `clip` exists for compatibility purposes, but it was never used in the original plugin.

        // TODO: test if it's a problem for compatibility that `super` is a reserved keyword
        let mvanalyse = Analyse::new(
            super_,
            blksize,
            blksizev,
            levels,
            search,
            searchparam,
            pelsearch,
            isb,
            lambda,
            chroma,
            delta,
            truemotion,
            lsad,
            plevel,
            global,
            pnew,
            pzero,
            pglobal,
            overlap,
            overlapv,
            divide,
            badsad,
            badrange,
            meander,
            trymany,
            fields,
            tff,
            search_coarse,
            dct,
        )?;

        Ok(Some(Box::new(mvanalyse)))
    }
}

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
        AnalyseFunction::new(),
        SuperFunction::new()
    ]
}
