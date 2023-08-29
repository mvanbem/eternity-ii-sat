use std::time::Instant;

use num_format::{SystemLocale, ToFormattedString};

use crate::mosaic::{ArrayMosaic, RectangularMosaic, SquareMosaic};
use crate::set::rectangle::RectangularMosaicSet;
use crate::set::square::SquareMosaicSet;

pub fn format_count(count: usize) -> String {
    count.to_formatted_string(&SystemLocale::default().unwrap())
}

pub fn format_ratio(numerator: usize, denominator: usize) -> String {
    format!("ratio {:.4}", numerator as f64 / denominator as f64,)
}

fn build_and_time<T>(
    title: &str,
    build: impl FnOnce() -> T,
    check: Option<impl FnOnce(&T)>,
    count: impl Fn(&T) -> usize,
    print_example: Option<impl FnOnce(&T)>,
) -> (TableRow, T) {
    use std::fmt::Write;

    if check.is_some() {
        println!("Building {title}...");
    } else {
        println!("Counting {title}...");
    }
    let start = Instant::now();
    let value = build();
    let build_time = Instant::now() - start;
    let count = count(&value);
    let count_string = format_count(count);
    println!(
        "* Found {count_string} mosaics in {}.{:03} s, average rate: {} mosaics/s",
        build_time.as_secs(),
        build_time.subsec_millis(),
        format_count(((count as f64) / build_time.as_secs_f64()).round() as usize),
    );

    let check_time = check.map(|check| {
        println!("Checking {title}...");
        let start = Instant::now();
        check(&value);
        Instant::now() - start
    });
    if let Some(print_example) = print_example {
        println!("Example element:");
        print_example(&value);
    }

    let mut times = format!(
        "{}.{:03} s {}",
        build_time.as_secs(),
        build_time.subsec_millis(),
        if check_time.is_some() {
            "build"
        } else {
            "count"
        },
    );
    if let Some(check_time) = check_time {
        write!(
            &mut times,
            ", {}.{:03} s check",
            check_time.as_secs(),
            check_time.subsec_millis()
        )
        .unwrap();
    }

    (
        TableRow {
            title: title.to_string(),
            count: count_string,
            notes: times,
        },
        value,
    )
}

fn print_rectangular_example<const W: usize, const H: usize, M: RectangularMosaic<W, H>>(
    set: &RectangularMosaicSet<W, H, M>,
) {
    print!("{}", set.iter_mosaics().next().unwrap().display(4));
}

fn print_square_example<const N: usize, M: SquareMosaic<N>>(set: &SquareMosaicSet<N, M>) {
    print!("{}", set.iter_mosaics().next().unwrap().display(4));
}

fn print_sample<const W: usize, const H: usize>((_, sample): &(usize, Option<ArrayMosaic<W, H>>)) {
    if let Some(sample) = sample.as_ref() {
        print!("{}", sample.display(4));
    } else {
        println!("    (zero elements)");
    }
}

#[derive(Default)]
pub struct Table {
    rows: Vec<TableRow>,
}

struct TableRow {
    title: String,
    count: String,
    notes: String,
}

impl Table {
    pub fn check_square<const N: usize, M: SquareMosaic<N>>(
        &mut self,
        title: &str,
        set: &SquareMosaicSet<N, M>,
    ) {
        println!("Checking {title}...");
        set.assert_distinct();
        println!("Passed! Example element:");
        print_square_example(set);

        self.rows.push(TableRow {
            title: title.to_string(),
            count: format_count(set.len()),
            notes: "".to_string(),
        });
    }

    pub fn track_build_rectangle<const W: usize, const H: usize, M: RectangularMosaic<W, H>>(
        &mut self,
        title: &str,
        build: impl FnOnce() -> RectangularMosaicSet<W, H, M>,
    ) -> RectangularMosaicSet<W, H, M> {
        let (row, result) = build_and_time(
            title,
            build,
            Some(&|set: &RectangularMosaicSet<W, H, M>| set.assert_distinct()),
            |set| set.len(),
            Some(&print_rectangular_example),
        );
        self.rows.push(row);
        result
    }

    pub fn track_build_square<const N: usize, M: SquareMosaic<N>>(
        &mut self,
        title: &str,
        build: impl FnOnce() -> SquareMosaicSet<N, M>,
    ) -> SquareMosaicSet<N, M> {
        let (row, result) = build_and_time(
            title,
            build,
            Some(&|set: &SquareMosaicSet<N, M>| set.assert_distinct()),
            |set| set.len(),
            Some(&print_square_example),
        );
        self.rows.push(row);
        result
    }

    pub fn track_count_and_sample<const W: usize, const H: usize>(
        &mut self,
        title: &str,
        build: impl FnOnce() -> (usize, Option<ArrayMosaic<W, H>>),
    ) -> usize {
        let (row, (result, _)) = build_and_time(
            title,
            build,
            None::<&dyn Fn(&_)>,
            |&(count, _)| count,
            Some(&print_sample),
        );
        self.rows.push(row);
        result
    }

    pub fn push(&mut self, title: &str, count: usize, notes: String) {
        self.rows.push(TableRow {
            title: title.to_string(),
            count: format_count(count),
            notes,
        });
    }

    pub fn push_hr(&mut self) {
        self.rows.push(TableRow {
            title: "----".to_string(),
            count: "".to_string(),
            notes: "".to_string(),
        });
    }

    pub fn print(&mut self) {
        let mut title_width = 0;
        let mut count_width = 0;
        let mut notes_width = 0;
        for row in &self.rows {
            title_width = title_width.max(row.title.len());
            count_width = count_width.max(row.count.len());
            notes_width = notes_width.max(row.notes.len());
        }

        println!();
        println!("Summary");
        println!("====");
        for row in &self.rows {
            println!(
                "{:<title_width$}  {:>count_width$}  {:<notes_width$}",
                row.title, row.count, row.notes,
            );
        }
        println!();

        self.push_hr();
    }
}
