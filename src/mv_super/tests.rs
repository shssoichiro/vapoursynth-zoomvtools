use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use vapoursynth::format::PresetFormat;

use super::*;
use crate::{
    params::{ReduceFilter, Subpel, SubpelMethod},
    tests::create_test_env,
};

const VALID_PRESETS: &[PresetFormat] = &[
    PresetFormat::Gray8,
    PresetFormat::Gray16,
    PresetFormat::YUV420P8,
    PresetFormat::YUV422P8,
    PresetFormat::YUV444P8,
    PresetFormat::YUV440P8,
    PresetFormat::YUV420P9,
    PresetFormat::YUV422P9,
    PresetFormat::YUV444P9,
    PresetFormat::YUV420P10,
    PresetFormat::YUV422P10,
    PresetFormat::YUV444P10,
    PresetFormat::YUV420P16,
    PresetFormat::YUV422P16,
    PresetFormat::YUV444P16,
    PresetFormat::YUV420P12,
    PresetFormat::YUV422P12,
    PresetFormat::YUV444P12,
    PresetFormat::YUV420P14,
    PresetFormat::YUV422P14,
    PresetFormat::YUV444P14,
];

#[test]
fn test_new_with_default_args() {
    let env = create_test_env(640, 480, PresetFormat::YUV420P8, 10).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let super_instance = Super::new(node, None, None, None, None, None, None, None, None).unwrap();

    assert_eq!(super_instance.hpad, 16);
    assert_eq!(super_instance.vpad, 16);
    assert_eq!(super_instance.pel, Subpel::Half);
    assert_eq!(super_instance.levels, 8);
    assert!(super_instance.chroma);
    assert_eq!(super_instance.sharp, SubpelMethod::Wiener);
    assert_eq!(super_instance.rfilter, ReduceFilter::Bilinear);
}

#[quickcheck]
fn test_new_with_specified_args(
    hpad: usize,
    vpad: usize,
    pel: u8,
    levels: u16,
    chroma: bool,
    sharp: u8,
    rfilter: u8,
) -> TestResult {
    if ![1, 2, 4].contains(&pel)
        || !(0..3).contains(&sharp)
        || !(0..5).contains(&rfilter)
        || hpad > 1024
        || vpad > 1024
        || levels > 64
    {
        return TestResult::discard();
    }

    let env = create_test_env(640, 480, PresetFormat::YUV420P8, 10).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let super_instance = Super::new(
        node,
        Some(hpad as i64),
        Some(vpad as i64),
        Some(pel as i64),
        Some(levels as i64),
        Some(chroma as i64),
        Some(sharp as i64),
        Some(rfilter as i64),
        None,
    )
    .unwrap();

    TestResult::from_bool(
        super_instance.hpad == hpad
            && super_instance.vpad == vpad
            && super_instance.pel == Subpel::try_from(pel as i64).unwrap()
            && super_instance.levels <= levels
            && super_instance.chroma == chroma
            && super_instance.sharp == SubpelMethod::try_from(sharp as i64).unwrap()
            && super_instance.rfilter == ReduceFilter::try_from(rfilter as i64).unwrap(),
    )
}
