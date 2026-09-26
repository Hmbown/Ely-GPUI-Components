use gpui::TestAppContext;

use super::{BarChart, LineChart, Series};
use crate::theme::Theme;

#[gpui::test]
fn a_chart_exports_as_svg_and_csv(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        let line = LineChart::new("sales", ["Jan", "Feb", "Mar"])
            .series(Series::new("Sales, net", [1.0, 3.0, 2.0]))
            .smooth();
        let svg = line.svg(400.0, 200.0, cx);
        assert!(svg.starts_with("<svg") && svg.ends_with("</svg>"));
        assert_eq!(svg.matches("<path").count(), 1, "one line, no area");
        assert!(svg.contains(" C"), "a smooth line curves");
        assert!(svg.contains(">Feb</text>"), "categories label the axis");
        assert_eq!(line.csv(), ",Jan,Feb,Mar\n\"Sales, net\",1,3,2");
        let bars = BarChart::new("share", ["A", "B"])
            .series(Series::new("Now", [2.0, 4.0]))
            .series(Series::new("Before", [1.0, 3.0]));
        let svg = bars.svg(400.0, 200.0, cx);
        assert_eq!(
            svg.matches("<rect").count(),
            5,
            "the page, then a bar per value"
        );
    });
}
