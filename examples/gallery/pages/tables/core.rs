use std::rc::Rc;

use ely_gpui_component::{
    buttons::SegmentedControl,
    data_display::Tone,
    forms::{SearchInput, TextInput},
    tables::{Aggregate, Cell, Column, DataTable, HeatmapTable, Row, Table},
    theme::{ActiveTheme, ControlSize, Density, Radius},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px, rems};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

pub fn plain(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "Table",
        "A header and rows, hairlines between. Nothing more.",
        cx,
    )
    .child(
        div().w(px(560.)).child(
            Table::new(["Name", "Role", "City"])
                .row(["Ada Lovelace", "Analyst", "London"])
                .row(["Grace Hopper", "Engineer", "Arlington"])
                .row(["Katherine Johnson", "Mathematician", "Hampton"]),
        ),
    )
}

const PEOPLE: [&str; 10] = [
    "Ada Lovelace",
    "Alan Turing",
    "Grace Hopper",
    "Katherine Johnson",
    "Tim Berners-Lee",
    "Radia Perlman",
    "Sophie Wilson",
    "Barbara Liskov",
    "Ken Thompson",
    "Margaret Hamilton",
];
const REGIONS: [&str; 4] = ["Europe", "Americas", "Asia", "Oceania"];

/// A steady pseudo-random series, so captures repeat.
fn noise(seed: u64) -> impl FnMut() -> f64 {
    let mut state = seed;
    move || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 33) as f64 / (1u64 << 31) as f64
    }
}

fn order_rows() -> Vec<Row> {
    let mut next = noise(7);
    (0..24)
        .map(|ix| {
            let (status, tone) = match (next() * 3.0) as u32 {
                0 => ("Live", Tone::Success),
                1 => ("Draft", Tone::Neutral),
                _ => ("Review", Tone::Warning),
            };
            let spark: Vec<f32> = (0..8).map(|_| next() as f32).collect();
            Row::new(
                format!("order-{ix}"),
                [
                    Cell::Person(PEOPLE[ix % PEOPLE.len()].into()),
                    Cell::Tag(status.into(), tone),
                    REGIONS[(next() * 4.0) as usize].into(),
                    (next() * 90_000.0 + 8_000.0).round().into(),
                    ((next() - 0.35) * 40.0).into(),
                    Cell::Spark(spark),
                    Cell::Progress(next() as f32),
                ],
            )
        })
        .collect()
}

fn order_columns() -> [Column; 7] {
    [
        Column::new("owner", "Owner"),
        Column::new("status", "Status").width(rems(6.)),
        Column::new("region", "Region").width(rems(6.)),
        Column::new("revenue", "Revenue")
            .end()
            .prefix("$")
            .aggregate(Aggregate::Sum)
            .width(rems(8.)),
        Column::new("growth", "Growth")
            .end()
            .decimals(1)
            .suffix("%")
            .scale()
            .aggregate(Aggregate::Average)
            .width(rems(6.)),
        Column::new("trend", "Trend").unsorted().width(rems(6.)),
        Column::new("quota", "Quota")
            .width(rems(9.))
            .aggregate(Aggregate::Average),
    ]
}

pub fn orders(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let search = window.use_keyed_state("orders-search", cx, TextInput::new);
    let query = search.read(cx).text().to_string();
    let picked = keep("orders-picked", Vec::<SharedString>::new, window, cx);
    let density = keep(
        "orders-density",
        || SharedString::from("standard"),
        window,
        cx,
    );
    let (chosen, store, dense, pick) = (
        picked.read(cx).clone(),
        picked.clone(),
        density.read(cx).clone(),
        density.clone(),
    );
    let spacing = match dense.as_ref() {
        "compact" => Density::Compact,
        "comfortable" => Density::Comfortable,
        _ => Density::Standard,
    };
    section(
        "DataTable / CellRenderer / RowSelection / TableDensity / AggregationFooter / ConditionalFormatting",
        "Press a header to sort, rising then falling. Search keeps rows that hold the text; boxes select, Shift takes a range. Cells draw people, tags, sparklines and progress; growth is tinted by where it sits; the footer sums and averages what the search keeps.",
        cx,
    )
    .child(
        div()
            .flex()
            .items_center()
            .justify_between()
            .w(px(900.))
            .pb_3()
            .child(div().w(px(260.)).child(SearchInput::new("orders-find", &search).size(ControlSize::Sm)))
            .child(
                SegmentedControl::new("orders-density", dense)
                    .segment("compact", "Compact", None)
                    .segment("standard", "Standard", None)
                    .segment("comfortable", "Comfortable", None)
                    .size(ControlSize::Sm)
                    .on_change(move |value, _, cx| set(&pick, value.clone(), cx)),
            ),
    )
    .child(probe(
        "orders",
        div().w(px(900.)).child(
            DataTable::new("orders", order_columns())
                .rows(order_rows())
                .query(query)
                .paged(8)
                .density(spacing)
                .selected(chosen.clone())
                .on_select(move |keys, _, cx| set(&store, keys.to_vec(), cx)),
        ),
    ))
    .child(Caption::new(format!("{} selected", chosen.len())))
}

pub fn long(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let rows = keep(
        "long-rows",
        || {
            let mut next = noise(11);
            Rc::new(
                (0..100_000)
                    .map(|ix| {
                        Row::new(
                            format!("long-{ix}"),
                            [
                                format!("Event {}", ix + 1).into(),
                                REGIONS[ix % 4].into(),
                                (next() * 1000.0).round().into(),
                            ],
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        },
        window,
        cx,
    )
    .read(cx)
    .clone();
    let theme = cx.theme();
    section(
        "VirtualTable",
        "A hundred thousand rows; only the ones in view are drawn, so it scrolls as lightly as ten.",
        cx,
    )
    .child(
        div()
            .w(px(560.))
            .h(px(300.))
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border)
            .child(
                DataTable::new(
                    "long",
                    [
                        Column::new("event", "Event"),
                        Column::new("region", "Region"),
                        Column::new("value", "Value").end().aggregate(Aggregate::Sum),
                    ],
                )
                .rows(rows)
                .virtualized()
                .density(Density::Compact)
                .size_full(),
            ),
    )
}

pub fn heatmap(cx: &mut App) -> impl IntoElement + use<> {
    let mut next = noise(3);
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let table = days.iter().enumerate().fold(
        HeatmapTable::new(["6", "9", "12", "15", "18", "21"]),
        |table, (day, name)| {
            let busy = if day >= 5 { 0.5 } else { 1.0 };
            table.row(
                *name,
                (0..6).map(|slot| {
                    (busy
                        * (40.0 + 60.0 * next())
                        * if (1..=4).contains(&slot) { 1.6 } else { 0.7 })
                    .round()
                }),
            )
        },
    );
    section(
        "HeatmapTable",
        "Visits by weekday and hour; each cell is shaded by where it sits between the quietest and the busiest.",
        cx,
    )
    .child(div().w(px(560.)).child(table))
    .child(Caption::new("ComparisonTable is Comparison, in Data Display."))
}
