let
  pkgs = import <nixpkgs> { };

  libraries = with pkgs; [
    at-spi2-atk
    atkmm
    cairo
    gdk-pixbuf
    glib
    gtk3
    harfbuzz
    librsvg
    libsoup_3
    pango
    webkitgtk_4_1
    openssl
  ];
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    gobject-introspection
    nodejs
    cargo-tauri
  ];

  buildInputs = libraries ++ [
    pkgs.gsettings-desktop-schemas
    pkgs.hicolor-icon-theme
    pkgs.gnome-themes-extra
    pkgs.libglvnd
  ];

  shellHook = ''
    # 1. Standard Library Path
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:/run/opengl-driver/lib:${pkgs.libglvnd}/lib:$LD_LIBRARY_PATH

    # 2. XDG Data Dirs (Themes)
    export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:${pkgs.gnome-themes-extra}/share:$XDG_DATA_DIRS

    # A. Disable Sandbox (Correct Variable)
    # Essential for preventing ghosting on NixOS
    # export WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1

    # B. Force Software Rendering (The "Squared Shadows" Fix)
    # This tells the underlying graphics library (Mesa) to stop trying
    # to use the confused Hardware/Nouveau drivers and just draw the pixels using the CPU.
    # This usually resolves alpha-transparency issues (squared corners).
    export LIBGL_ALWAYS_SOFTWARE=1

    # --- DEBUGGING (Uncomment these if it still fails) ---
    # export G_MESSAGES_DEBUG=all
    # export WEBKIT_DEBUG=compositing,layers

    # surrealdb-librocksdb-sys does not compile without this one
    export LIBCLANG_PATH=/nix/store/19mjhjglq0g1qrnyr7prbi6xxl1ghsr3-user-environment/lib
  '';
}
