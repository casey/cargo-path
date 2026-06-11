watch +args='ltest':
  cargo watch --clear --exec '{{ args }}'

run dependency:
  cargo run path {{ dependency }}

clippy: (watch 'lclippy --tests --all --all-targets -- --deny warnings')

check: (watch 'lcheck --tests --all --all-targets')

test: (watch 'ltest --all --all-targets')

outdated:
  cargo outdated -R

unused:
  cargo +nightly udeps --workspace
