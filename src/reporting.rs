use serde_json::Value;
use std::fmt;
use tabled::{Table, Tabled};
use tabled::settings::{Format, Modify, object::Segment};
use tabled::settings::style::Style;
use colored::*;

/// Helper type to display Option<f64> rounded or as "-" if None
#[derive(Clone)]
pub struct DisplayableF64(pub Option<f64>);

impl fmt::Display for DisplayableF64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(val) => write!(f, "{:.3}", val),
            None => write!(f, "-"),
        }
    }
}

/// One row in the final table: one position or "AVG" summary
#[derive(Tabled)]
pub struct PositionRow {
    #[tabled(rename = "Pos")]
    pub pos: String,
    #[tabled(rename = "Quality")]
    pub quality: DisplayableF64,
    #[tabled(rename = "GC%")]
    pub gc: DisplayableF64,
    #[tabled(rename = "A%")]
    pub a: DisplayableF64,
    #[tabled(rename = "C%")]
    pub c: DisplayableF64,
    #[tabled(rename = "G%")]
    pub g: DisplayableF64,
    #[tabled(rename = "T%")]
    pub t: DisplayableF64,
    #[tabled(rename = "N%")]
    pub n: DisplayableF64,
}

#[derive(Tabled)]
pub struct LengthRow {
    #[tabled(rename = "Length")]
    pub length: String,
    #[tabled(rename = "Count")]
    pub count: String,
    #[tabled(rename = "Percent")]
    pub percent: String,
}


/// Main function to print combined table
pub fn print_combined_table(json: &Value) {
    let base_q = json.get("average_base_quality_per_position").and_then(|v| v.as_array());
    let gc = json.get("average_gc_content_per_position").and_then(|v| v.as_array());
    let comp = json.get("base_composition_per_position").and_then(|v| v.as_array());

    let len = base_q.as_ref().map(|v| v.len())
        .or_else(|| gc.as_ref().map(|v| v.len()))
        .or_else(|| comp.as_ref().map(|v| v.len()))
        .unwrap_or(0);

    let mut rows = Vec::new();

    for i in 0..len {
        let quality = base_q.and_then(|v| v.get(i)).and_then(|x| x.as_f64());
        let gc_val = gc.and_then(|v| v.get(i)).and_then(|x| x.as_f64());
        let (a, c, g, t, n) = if let Some(Some(obj)) = comp.as_ref()
            .map(|v| v.get(i))
            .map(|o| o.and_then(|o| o.as_object())) {
            (
                obj.get("A").and_then(|v| v.as_f64()),
                obj.get("C").and_then(|v| v.as_f64()),
                obj.get("G").and_then(|v| v.as_f64()),
                obj.get("T").and_then(|v| v.as_f64()),
                obj.get("N").and_then(|v| v.as_f64()),
            )
        } else {
            (None, None, None, None, None)
        };

        rows.push(PositionRow {
            pos: i.to_string().blue().to_string(),
            quality: DisplayableF64(quality),
            gc: DisplayableF64(gc_val),
            a: DisplayableF64(a),
            c: DisplayableF64(c),
            g: DisplayableF64(g),
            t: DisplayableF64(t),
            n: DisplayableF64(n),
        });
    }

    // Add AVG summary row (per-read metrics)
    let avg = PositionRow {
        pos: "AVG".bold().yellow().to_string(),
        quality: DisplayableF64(json.get("average_read_quality").and_then(|v| v.as_f64())),
        gc: DisplayableF64(json.get("average_gc_content_per_read").and_then(|v| v.as_f64())),
        a: DisplayableF64(json.get("average_base_composition_per_read").and_then(|o| o.get("A")).and_then(|v| v.as_f64())),
        c: DisplayableF64(json.get("average_base_composition_per_read").and_then(|o| o.get("C")).and_then(|v| v.as_f64())),
        g: DisplayableF64(json.get("average_base_composition_per_read").and_then(|o| o.get("G")).and_then(|v| v.as_f64())),
        t: DisplayableF64(json.get("average_base_composition_per_read").and_then(|o| o.get("T")).and_then(|v| v.as_f64())),
        n: DisplayableF64(json.get("average_base_composition_per_read").and_then(|o| o.get("N")).and_then(|v| v.as_f64())),
    };

    rows.push(avg);

    let mut table = Table::new(rows);
    table
        .with(Style::modern())
        .with(Modify::new(Segment::all()).with(Format::content(|s| s.to_string())));

    println!("\nStatistics Table:\n");
    println!("{table}");

}

pub fn print_length_distribution(json: &Value) {
    if let Some(dist) = json.get("read_length_distribution").and_then(|v| v.as_object()) {
        let mut rows = Vec::new();

        let mut lengths: Vec<_> = dist.iter().collect();
        lengths.sort_by_key(|(k, _)| k.parse::<usize>().unwrap_or(0)); // Sortieren nach numerischer Länge

        for (len, val) in lengths {
            let count = val.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            let percent = val.get("percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
            rows.push(LengthRow {
                length: len.clone().cyan().to_string(),
                count: count.to_string(),
                percent: format!("{:.2}%", percent * 100.0),
            });
        }

        let mut table = Table::new(rows);
        table
            .with(Style::modern())
            .with(Modify::new(Segment::all()).with(Format::content(|s| s.to_string())));

        println!("\nRead Length Distribution:\n");
        println!("{table}");
    }
}
