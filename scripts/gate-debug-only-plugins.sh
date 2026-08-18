#!/usr/bin/env bash
# Wrap dev-dependency plugin imports and registrations in `#if DEBUG`.
#
# Usage: scripts/gate-debug-only-plugins.sh <GeneratedPluginRegistrant.m>
#
# `integration_test` powers the on-device suites in app/integration_test and must
# never ship. app/ios/Podfile installs its pod for the Debug configuration only,
# but Flutter regenerates GeneratedPluginRegistrant on every build and does not
# filter dev dependencies on Apple platforms (flutter/flutter#163874). Without
# this gate a Profile or Release compile would import a module that is not there.
#
# Runs from the Runner target's Flutter build phase, before Compile Sources.
set -euo pipefail

registrant="${1:?usage: $0 <GeneratedPluginRegistrant.m>}"
[[ -f "$registrant" ]] || exit 0

DEBUG_ONLY_PLUGINS=(integration_test)

python3 - "$registrant" "${DEBUG_ONLY_PLUGINS[@]}" <<'PY'
import re
import sys

path, *plugins = sys.argv[1:]
with open(path) as handle:
    source = handle.read()
patched = source

for plugin in plugins:
    if '#if DEBUG\n#if __has_include(<%s/' % plugin in patched:
        continue  # already gated
    imports = re.compile(
        r'^#if __has_include\(<%s/(\w+)\.h>\).*?^#endif\n' % re.escape(plugin),
        re.S | re.M,
    )
    found = imports.search(patched)
    if found is None:
        continue
    plugin_class = found.group(1)
    patched = imports.sub(
        lambda block: '#if DEBUG\n%s#endif\n' % block.group(0), patched, count=1
    )
    registration = re.compile(
        r'^[ \t]*\[%s registerWithRegistrar:[^\n]*\n' % re.escape(plugin_class),
        re.M,
    )
    patched = registration.sub(
        lambda line: '#if DEBUG\n%s#endif\n' % line.group(0), patched, count=1
    )

if patched != source:
    with open(path, 'w') as handle:
        handle.write(patched)
    print('gated debug-only plugins in %s' % path)
PY
