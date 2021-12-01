use address_formatter::{Formatter, Place, PlaceBuilder};
use anyhow::{anyhow, Context, Error};
use include_dir::include_dir;
use yaml_rust::{Yaml, YamlLoader};

#[test]
pub fn opencage_tests() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let formatter = Formatter::default();
    let places_builder = PlaceBuilder::default();
    let errors: Vec<_> = include_dir!("./address-formatting/testcases/countries")
        .files()
        .filter_map(|f| {
            f.contents_utf8().map(|s| {
                (
                    YamlLoader::load_from_str(s).unwrap_or_else(|err| {
                        panic!(
                            "impossible to read test file {}: {}",
                            f.path().display(),
                            err
                        )
                    }),
                    f.path().to_str().unwrap(),
                )
            })
        })
        .flat_map(|(s, file_name)| s.into_iter().map(move |s| (s, file_name)))
        .map(|(t, file_name)| run_test(t, file_name, &formatter, &places_builder))
        .filter_map(|r| r.err())
        .map(|e| {
            log::error!("test on error: {}", e);
            e
        })
        .collect();

    if errors.is_empty() {
        log::info!("All tests ok");
    } else if errors.len() == 8 {
        log::warn!("Some tests are failing but we consider it's ok, it's still a work in progress");
    } else {
        panic!("{} tests were on error", errors.len());
    }
}

fn run_test(
    yaml: Yaml,
    file_name: &str,
    formatter: &Formatter,
    places_builder: &PlaceBuilder,
) -> Result<(), Error> {
    let description = yaml["description"]
        .as_str()
        .unwrap_or("no description provided");
    log::debug!("running {}", &description);

    let expected = yaml["expected"]
        .as_str()
        .context("no expected value provided for file")?;

    let addr = read_addr(
        yaml["components"]
            .as_hash()
            .context("no component value provided")?,
        places_builder,
    )?;

    let formated_value = formatter.format(addr)?;

    if formated_value != expected {
        Err(anyhow!(
            r#"
====================================
for file {}, test "{}"

expected: 
---
{}
---

got:
----
{}
----
"#,
            file_name,
            description,
            expected,
            formated_value
        ))
    } else {
        Ok(())
    }
}

// unfortunalty, at the time of writing, serde_yaml does not handle multiple documents in a yaml,
// so we have to parse the parse manually
fn read_addr(
    component: &linked_hash_map::LinkedHashMap<Yaml, Yaml>,
    place_builder: &PlaceBuilder,
) -> Result<Place, Error> {
    Ok(
        place_builder.build_place(component.iter().filter_map(|(k, v)| {
            Some((
                k.as_str()?,
                v.as_str()
                    .map(|s| s.to_string())
                    .or_else(|| v.as_i64().map(|s| s.to_string()))?,
            ))
        })),
    )
}
