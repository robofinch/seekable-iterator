# Not implemented: (test-all) (build-all) (bench-all)

list:
    just --list

# ================================================================
#   Example `.vscode/settings.json` for `rust-analyzer`:
# ================================================================

# {
#     "rust-analyzer.check.overrideCommand": [
#         "just",
#         "on-save",
#     ],
#     "rust-analyzer.checkOnSave": true,
# }

# ================================================================
#   Smaller scripts
# ================================================================

# Run ripgrep, but don't return an error if nothing matched.
[group("ripgrep")]
rg-maybe-no-match *args:
    @rg {{ args }} || [ $? -eq 1 ]

# Find lines not ending in a comma, where the next line starts with `]`, `)`, or `>`.
[group("ripgrep")]
find-possible-missing-commas: \
    (rg-maybe-no-match ''' -U '[^,]\n[ ]*\]' ''') \
    (rg-maybe-no-match ''' -U '[^,]\n[ ]*\)' ''') \
    (rg-maybe-no-match ''' -U '[^,]\n[ ]*>' ''')

# Find any `#[allow(...)]` attribute, or to be precise, find `[allow(`.
[group("ripgrep")]
find-allow-attributes: (rg-maybe-no-match '"\[allow\("')

# Find any possible sites of unsafe code.
[group("ripgrep")]
find-unsafe-code: (rg-maybe-no-match '"unsafe_code|unsafe"')

# ================================================================
#   Check and Clippy
# ================================================================

plus-msrv := '+1.85'

check *args:
    cargo +stable hack clippy --feature-powerset {{args}}
    cargo +nightly hack clippy --feature-powerset {{args}}
    cargo {{plus-msrv}} hack clippy --feature-powerset {{args}}

clippy *args:
    cargo +stable hack clippy --feature-powerset {{args}}
    RUSTFLAGS="-Zcrate-attr=feature(\
                    strict_provenance_lints,\
                    multiple_supertrait_upcastable,\
                    must_not_suspend,\
                    non_exhaustive_omitted_patterns_lint,\
                    supertrait_item_shadowing,\
                    unqualified_local_imports\
                ) \
                -Wfuzzy_provenance_casts \
                -Wlossy_provenance_casts \
                -Wmultiple_supertrait_upcastable \
                -Wmust_not_suspend \
                -Wnon_exhaustive_omitted_patterns \
                -Wsupertrait_item_shadowing_definition \
                -Wsupertrait_item_shadowing_usage \
                -Wunqualified_local_imports" cargo +nightly hack clippy --feature-powerset {{args}}
    cargo {{plus-msrv}} hack clippy --feature-powerset {{args}}

[group("on-save")]
on-save: (clippy "--message-format=json")
