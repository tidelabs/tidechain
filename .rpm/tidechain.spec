%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: tidechain
Summary: Implementation of TIDE Chain node in Rust based on the Substrate framework.
Version: @@VERSION@@
Release: @@RELEASE@@%{?dist}
License: GPLv3+
Group: Applications/System
Source0: %{name}-%{version}.tar.gz

Requires: systemd, shadow-utils
Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%post
config_file="/etc/default/tidechain"
getent group tidechain >/dev/null || groupadd -r tidechain
getent passwd tidechain >/dev/null || \
    useradd -r -g tidechain -d /home/tidechain -m -s /sbin/nologin \
    -c "User account for running tidechain as a service" tidechain
if [ ! -e "$config_file" ]; then
    echo 'TIDECHAIN_CLI_ARGS=""' > /etc/default/tidechain
fi
exit 0

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
/usr/lib/systemd/system/tidechain.service