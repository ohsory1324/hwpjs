/// 옛 판(HWP 5.0.0.x ~ 5.0.3.x) 문서의 짧은 레코드 처리 테스트
/// Tests for short records written by older HWP 5.0.x versions
///
/// fixture는 실제 지방의회 회의록 부록 파일에서 가져왔습니다.
/// - `tests/fixtures/old-records/*.bin`: 레코드 데이터(헤더 제외)만 잘라낸 것
/// - `tests/fixtures/old-records/*.hwp`: 작은 원본 파일 전체
/// 하위 폴더에 두어 스냅샷 테스트(`find_all_hwp_files`)의 대상이 되지 않게 했습니다.
mod common;

use hwp_core::document::bodytext::{EqEdit, ParaHeader, ShapeComponentPicture, Table};
use hwp_core::viewer::markdown::MarkdownOptions;
use hwp_core::{HwpParser, ParaShape};
use std::path::PathBuf;

fn old_fixture(name: &str) -> Vec<u8> {
    let dir = common::find_fixtures_dir().expect("fixtures dir");
    let path: PathBuf = dir.join("old-records").join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

const V5_0_0_6: u32 = 0x05000006;
const V5_0_3_3: u32 = 0x05000303;

#[test]
fn para_shape_42_bytes_from_5_0_0_6() {
    let data = old_fixture("para_shape_5.0.0.6_42b.bin");
    assert_eq!(data.len(), 42);
    let ps = ParaShape::parse(&data, V5_0_0_6).expect("42-byte PARA_SHAPE should parse");
    // 레코드에 없는 뒤쪽 테두리 간격은 0 / Missing trailing border spacing defaults to 0
    assert_eq!(ps.border_spacing_top, 0);
    assert_eq!(ps.border_spacing_bottom, 0);
    assert!(ps.attributes2.is_none());
    assert!(ps.attributes3.is_none());
    assert!(ps.line_spacing.is_none());
}

#[test]
fn para_shape_shorter_than_42_bytes_is_error() {
    let data = old_fixture("para_shape_5.0.0.6_42b.bin");
    assert!(ParaShape::parse(&data[..41], V5_0_0_6).is_err());
}

#[test]
fn para_header_22_bytes_in_5_0_3_3_file() {
    // 스펙상 5.0.3.2 이상은 24바이트지만 실제 5.0.3.3 파일에 22바이트가 있음
    let data = old_fixture("para_header_5.0.3.3_22b.bin");
    assert_eq!(data.len(), 22);
    let ph = ParaHeader::parse(&data, V5_0_3_3).expect("22-byte PARA_HEADER should parse");
    assert_eq!(ph.text_char_count, 39);
    assert_eq!(ph.para_shape_id, 21);
    assert_eq!(ph.char_shape_count, 2);
    assert_eq!(ph.line_align_count, 1);
    assert_eq!(ph.section_merge, None);
}

#[test]
fn para_header_24_bytes_still_reads_section_merge() {
    let mut data = old_fixture("para_header_5.0.3.3_22b.bin");
    data.extend_from_slice(&[0x01, 0x00]);
    let ph = ParaHeader::parse(&data, V5_0_3_3).unwrap();
    assert_eq!(ph.section_merge, Some(1));
}

#[test]
fn picture_73_bytes_from_5_0_0_6() {
    let data = old_fixture("picture_5.0.0.6_73b.bin");
    assert_eq!(data.len(), 73);
    let pic = ShapeComponentPicture::parse(&data).expect("73-byte picture should parse");
    assert_eq!(pic.picture_info.bindata_id, 1);
    assert_eq!(pic.border_opacity, 0);
    assert_eq!(pic.instance_id, 0);
    assert!(pic.effect_data.is_empty());
}

#[test]
fn picture_74_bytes_from_5_0_2_4() {
    let data = old_fixture("picture_5.0.2.4_74b.bin");
    assert_eq!(data.len(), 74);
    let pic = ShapeComponentPicture::parse(&data).expect("74-byte picture should parse");
    assert_eq!(pic.picture_info.bindata_id, 1);
    assert_eq!(pic.instance_id, 0);
    assert!(pic.effect_data.is_empty());
}

#[test]
fn eqedit_5_1_0_1_with_length_prefixed_strings() {
    let data = old_fixture("eqedit_5.1.0.1_104b.bin");
    let eq = EqEdit::parse(&data).expect("EQEDIT should parse");
    assert_eq!(eq.script_length, 15);
    assert_eq!(eq.script, "{10} over {100}");
    assert_eq!(u32::from(eq.character_size), 1000);
    assert_eq!(eq.base_line, 66);
    assert_eq!(eq.version_info, "Equation Version 60");
    assert_eq!(eq.font_name, "HYhwpEQ");
}

#[test]
fn eqedit_5_1_0_1_with_empty_version() {
    let data = old_fixture("eqedit_5.1.0.1_66b.bin");
    let eq = EqEdit::parse(&data).expect("EQEDIT should parse");
    assert_eq!(eq.script, "1 OVER 1000\r\n\r\n");
    assert_eq!(eq.version_info, "");
    assert_eq!(eq.font_name, "HYhwpEQ");
}

#[test]
fn eqedit_5_0_0_6_without_trailing_strings() {
    let data = old_fixture("eqedit_5.0.0.6_130b.bin");
    let eq = EqEdit::parse(&data).expect("EQEDIT should parse");
    assert_eq!(eq.script_length, 56);
    assert_eq!(
        eq.script,
        " {111,436백만원(자체재원)} over {270,454백만원(일반회계)} TIMES 100 \r\n"
    );
    assert_eq!(u32::from(eq.character_size), 1200);
    assert_eq!(eq.base_line, 67);
    assert_eq!(eq.version_info, "");
    assert_eq!(eq.font_name, "");
}

#[test]
fn eqedit_5_0_2_4_with_only_version_length() {
    let data = old_fixture("eqedit_5.0.2.4_132b.bin");
    let eq = EqEdit::parse(&data).expect("EQEDIT should parse");
    assert_eq!(eq.script_length, 56);
    assert_eq!(eq.version_info, "");
    assert_eq!(eq.font_name, "");
}

#[test]
fn eqedit_truncated_script_does_not_fail() {
    // 스크립트 중간에서 잘린 레코드도 문서 전체를 버리지 않음
    let data = old_fixture("eqedit_5.1.0.1_104b.bin");
    let eq = EqEdit::parse(&data[..6 + 2 * 5 + 1]).expect("truncated EQEDIT should parse");
    assert_eq!(eq.script, "{10} ");
    assert_eq!(eq.version_info, "");
    assert_eq!(eq.font_name, "");
}

#[test]
fn table_without_zone_info_from_5_0_0_6() {
    let data = old_fixture("table_5.0.0.6_22b.bin");
    let table = Table::parse(&data, V5_0_0_6).expect("table without zone info should parse");
    assert_eq!(table.attributes.row_count, 1);
    assert_eq!(table.attributes.col_count, 1);
    assert!(table.attributes.zones.is_empty());
}

fn markdown_of(name: &str) -> String {
    let data = old_fixture(name);
    let document = HwpParser::new()
        .parse(&data)
        .unwrap_or_else(|e| panic!("{} should parse: {}", name, e));
    let options = MarkdownOptions {
        image_output_dir: None,
        use_html: Some(true),
        include_version: Some(true),
        include_page_info: Some(false),
    };
    document.to_markdown(&options)
}

#[test]
fn whole_file_5_0_0_6_nowon() {
    // PARA_SHAPE 42바이트 + PARA_HEADER 22바이트
    let md = markdown_of("nowon-5.0.0.6.hwp");
    assert!(md.contains("예산결산특별위원회구성결의안"), "{}", md);
    assert!(md.contains("위원수는 5인으로 한다"), "{}", md);
}

#[test]
fn whole_file_5_0_3_3_seongbuk() {
    // PARA_HEADER 22바이트 (판 번호는 5.0.3.3)
    let md = markdown_of("seongbuk-5.0.3.3.hwp");
    assert!(md.contains("공무국외여행 규칙안 검토보고"), "{}", md);
    assert!(md.contains("제안경위"), "{}", md);
}

#[test]
fn whole_file_5_0_0_6_seongbuk_with_tables() {
    // PARA_SHAPE 42바이트 + 영역 정보 없는 표 레코드
    let md = markdown_of("seongbuk-5.0.0.6.hwp");
    assert!(md.contains("제 안 사 항"), "{}", md);
    assert!(md.contains("행정기구설치 조례"), "{}", md);
}
