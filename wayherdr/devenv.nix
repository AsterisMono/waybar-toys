{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/packages/
  packages = [ pkgs.git ];

  # https://devenv.sh/languages/
  languages.rust.enable = true;

  # https://devenv.sh/scripts/
  scripts.build.exec = "cargo build";
  scripts.test.exec = "cargo test";

  # https://devenv.sh/basics/
  enterShell = ''
    cargo build   # compile wayherdr
  '';

  # https://devenv.sh/tests/
  enterTest = ''
    cargo test
  '';
}
