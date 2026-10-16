Name:           pomato
Version:        0.1.0
Release:        1%{?dist}
Summary:        Simple Pomodoro timer for GNOME
License:        GPL-3.0-or-later
URL:            https://github.com/pomato/pomato
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  pkgconfig(gtk4) >= 4.20
BuildRequires:  pkgconfig(libadwaita-1) >= 1.9
BuildRequires:  pkgconfig(glib-2.0)
BuildRequires:  desktop-file-utils
BuildRequires:  libappstream-glib

Requires:       gtk4%{?_isa}
Requires:       libadwaita%{?_isa}
Recommends:     gnome-shell-extension-appindicator

%description
Pomato is a simple Pomodoro timer for the GNOME desktop.
It supports focus sessions, short and long breaks, desktop
notifications, a StatusNotifierItem tray icon, and GSettings-based
preferences.

%prep
%autosetup -n %{name}-%{version}

# Offline build with vendored crates (created by `cargo vendor`).
%cargo_prep

%generate_buildrequires
%cargo_generate_buildrequires

%build
%cargo_build

%install
%cargo_install

install -Dpm0644 data/io.github.pomato.Pomato.desktop \
  %{buildroot}%{_datadir}/applications/io.github.pomato.Pomato.desktop

install -Dpm0644 data/io.github.pomato.Pomato.gschema.xml \
  %{buildroot}%{_datadir}/glib-2.0/schemas/io.github.pomato.Pomato.gschema.xml

install -Dpm0644 data/io.github.pomato.Pomato.service \
  %{buildroot}%{_datadir}/dbus-1/services/io.github.pomato.Pomato.service

for sz in 256 128 64 48 32; do
  install -Dpm0644 data/icons/hicolor/${sz}x${sz}/apps/pomato.png \
    %{buildroot}%{_datadir}/icons/hicolor/${sz}x${sz}/apps/pomato.png
done

%check
desktop-file-validate %{buildroot}%{_datadir}/applications/io.github.pomato.Pomato.desktop
%cargo_test

%post
%glib2_gsettings_schema_post
%icon_theme_cache_post

%postun
%glib2_gsettings_schema_postun
%icon_theme_cache_postun

%files
%license LICENSE
%doc README.md
%{_bindir}/pomato
%{_datadir}/applications/io.github.pomato.Pomato.desktop
%{_datadir}/glib-2.0/schemas/io.github.pomato.Pomato.gschema.xml
%{_datadir}/dbus-1/services/io.github.pomato.Pomato.service
%{_datadir}/icons/hicolor/*/apps/pomato.png

%changelog
* Thu Oct 16 2026 Pomato Maintainers <pomato@localhost> - 0.1.0-1
- Initial package
