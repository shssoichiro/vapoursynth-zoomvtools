// Performance
#![warn(clippy::clear_with_drain)]
#![warn(clippy::format_collect)]
#![warn(clippy::format_push_string)]
#![warn(clippy::imprecise_flops)]
#![warn(clippy::inefficient_to_string)]
#![warn(clippy::inline_always)]
#![warn(clippy::iter_with_drain)]
#![warn(clippy::large_include_file)]
#![warn(clippy::large_types_passed_by_value)]
#![deny(clippy::linkedlist)]
// Can result in worse code generation: https://github.com/rust-lang/rust-clippy/issues/14944
#![allow(clippy::manual_div_ceil)]
#![warn(clippy::mutex_atomic)]
#![warn(clippy::mutex_integer)]
#![warn(clippy::naive_bytecount)]
#![warn(clippy::needless_bitwise_bool)]
#![warn(clippy::needless_collect)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::non_std_lazy_statics)]
#![warn(clippy::non_zero_suggestions)]
#![warn(clippy::or_fun_call)]
#![warn(clippy::rc_buffer)]
#![warn(clippy::redundant_clone)]
#![warn(clippy::ref_option)]
#![warn(clippy::set_contains_or_insert)]
#![warn(clippy::stable_sort_primitive)]
#![warn(clippy::string_lit_chars_any)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::trivial_regex)]
#![warn(clippy::trivially_copy_pass_by_ref)]
#![warn(clippy::unnecessary_box_returns)]
#![warn(clippy::unnecessary_join)]
#![warn(clippy::unused_async)]
#![warn(clippy::verbose_file_reads)]
// Readability/Code Intention
#![warn(clippy::checked_conversions)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::cloned_instead_of_copied)]
#![warn(clippy::enum_glob_use)]
#![warn(clippy::equatable_if_let)]
#![warn(clippy::filter_map_next)]
#![warn(clippy::flat_map_option)]
#![warn(clippy::if_then_some_else_none)]
#![warn(clippy::implicit_clone)]
#![warn(clippy::inconsistent_struct_constructor)]
#![warn(clippy::invalid_upcast_comparisons)]
#![warn(clippy::iter_filter_is_ok)]
#![warn(clippy::iter_filter_is_some)]
#![warn(clippy::iter_on_empty_collections)]
#![warn(clippy::iter_on_single_items)]
#![warn(clippy::macro_use_imports)]
#![warn(clippy::manual_assert)]
#![warn(clippy::manual_instant_elapsed)]
#![warn(clippy::manual_is_power_of_two)]
#![warn(clippy::manual_is_variant_and)]
#![warn(clippy::manual_let_else)]
#![warn(clippy::manual_string_new)]
#![warn(clippy::map_unwrap_or)]
#![warn(clippy::map_with_unused_argument_over_ranges)]
#![warn(clippy::match_bool)]
#![warn(clippy::mod_module_files)]
#![warn(clippy::needless_continue)]
#![warn(clippy::needless_pass_by_ref_mut)]
#![warn(clippy::option_as_ref_cloned)]
#![warn(clippy::option_if_let_else)]
#![warn(clippy::pathbuf_init_then_push)]
#![warn(clippy::precedence_bits)]
#![warn(clippy::range_minus_one)]
#![warn(clippy::range_plus_one)]
#![warn(clippy::redundant_test_prefix)]
#![warn(clippy::ref_option_ref)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::tests_outside_test_module)]
#![warn(clippy::transmute_ptr_to_ptr)]
#![warn(clippy::unused_peekable)]
#![warn(clippy::unused_rounding)]
#![warn(clippy::verbose_bit_mask)]
#![warn(clippy::zero_sized_map_values)]
// Correctness/Safety
#![warn(clippy::case_sensitive_file_extension_comparisons)]
#![deny(clippy::cfg_not_test)]
#![warn(clippy::collection_is_never_read)]
#![warn(clippy::create_dir)]
#![warn(clippy::dbg_macro)]
#![deny(clippy::debug_assert_with_mut_call)]
#![deny(clippy::expl_impl_clone_on_copy)]
#![warn(clippy::filetype_is_file)]
#![warn(clippy::future_not_send)]
#![warn(clippy::ignore_without_reason)]
#![warn(clippy::infinite_loop)]
#![warn(clippy::large_futures)]
#![warn(clippy::large_stack_arrays)]
#![warn(clippy::large_stack_frames)]
#![warn(clippy::manual_midpoint)]
#![warn(clippy::maybe_infinite_iter)]
#![warn(clippy::mem_forget)]
#![warn(clippy::mismatching_type_param_order)]
#![warn(clippy::mixed_read_write_in_expression)]
#![warn(clippy::mut_mut)]
#![deny(clippy::non_send_fields_in_send_ty)]
#![warn(clippy::path_buf_push_overwrite)]
#![warn(clippy::rc_mutex)]
#![warn(clippy::read_zero_byte_vec)]
#![deny(clippy::significant_drop_in_scrutinee)]
#![warn(clippy::str_split_at_newline)]
#![warn(clippy::string_slice)]
#![warn(clippy::suspicious_operation_groupings)]
#![warn(clippy::suspicious_xor_used_as_pow)]
#![warn(clippy::transmute_undefined_repr)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::unnecessary_debug_formatting)]
#![warn(clippy::unwrap_used)]
// Annoyances
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::uninlined_format_args)]
#![allow(unsafe_op_in_unsafe_fn)]

use anyhow::Error;
use mv_analyse::Analyse;
use mv_super::Super;
use vapoursynth::{
    api::API,
    core::CoreRef,
    export_vapoursynth_plugin,
    make_filter_function,
    node::Node,
    plugins::{Filter, FilterArgument, Metadata},
};

#[cfg(test)]
#[macro_use]
mod tests;

#[cfg(feature = "bench")]
pub mod average;
#[cfg(feature = "bench")]
pub mod dct;
#[cfg(feature = "bench")]
pub mod group_of_planes;
#[cfg(feature = "bench")]
pub mod mv;
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
pub mod plane_of_blocks;
#[cfg(feature = "bench")]
pub mod reduce;
#[cfg(feature = "bench")]
pub mod refine;
#[cfg(feature = "bench")]
pub mod util;

#[cfg(not(feature = "bench"))]
mod average;
#[cfg(not(feature = "bench"))]
mod dct;
#[cfg(not(feature = "bench"))]
mod group_of_planes;
#[cfg(not(feature = "bench"))]
mod mv;
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
mod plane_of_blocks;
#[cfg(not(feature = "bench"))]
mod reduce;
#[cfg(not(feature = "bench"))]
mod refine;
mod simd;
#[cfg(not(feature = "bench"))]
mod util;

pub const PLUGIN_IDENTIFIER: &str = "com.soichiro.zoomvtools";
pub const PLUGIN_NAME: &str = "zoomvtools";

make_filter_function! {
    AnalyseFunction, "Analyse"
    fn create_analyse<'core>(
        _api: API,
        _core: CoreRef<'core>,
        super_clip: Node<'core>,
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
        clip: Option<Node<'core>>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        // `opt` exists for compatibility purposes, but will not be used.
        // `clip` exists for compatibility purposes, but it was never used in the original plugin.
        let mvanalyse = Analyse::new(
            super_clip,
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
