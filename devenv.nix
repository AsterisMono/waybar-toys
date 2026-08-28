{ pkgs, ... }:

{
  packages = [ pkgs.git ];

  languages.rust = {
    enable = true;
    channel = "stable";
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
  };

  scripts = {
    check.exec = "cargo check --workspace";
    test.exec = "cargo test --workspace";
  };

  enterTest = ''
    cargo test --workspace
  '';
}
