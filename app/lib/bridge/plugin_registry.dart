import 'dart:convert';

/// Capability codes mirror `tw_plugin::Capability` (F26.S3).
enum PluginCapability {
  documentRead,
  documentEdit,
  documentSuggest,
  uiSidebar,
  network,
  storage,
  filesystemRead,
}

extension PluginCapabilityWire on PluginCapability {
  String get wire => switch (this) {
        PluginCapability.documentRead => 'document.read',
        PluginCapability.documentEdit => 'document.edit',
        PluginCapability.documentSuggest => 'document.suggest',
        PluginCapability.uiSidebar => 'ui.sidebar',
        PluginCapability.network => 'network',
        PluginCapability.storage => 'storage',
        PluginCapability.filesystemRead => 'filesystem.read',
      };

  static PluginCapability? parse(String value) {
    return switch (value) {
      'document.read' || 'document_read' => PluginCapability.documentRead,
      'document.edit' || 'document_edit' => PluginCapability.documentEdit,
      'document.suggest' || 'document_suggest' => PluginCapability.documentSuggest,
      'ui.sidebar' || 'ui_sidebar' => PluginCapability.uiSidebar,
      'network' => PluginCapability.network,
      'storage' => PluginCapability.storage,
      'filesystem.read' || 'filesystem_read' => PluginCapability.filesystemRead,
      _ => null,
    };
  }
}

class PluginManifestInfo {
  const PluginManifestInfo({
    required this.id,
    required this.name,
    required this.version,
    required this.capabilities,
    required this.granted,
    this.enabled = true,
  });

  final String id;
  final String name;
  final String version;
  final List<PluginCapability> capabilities;
  final List<PluginCapability> granted;
  final bool enabled;

  bool hasCapability(PluginCapability cap) => granted.contains(cap);

  Map<String, dynamic> toJson() => {
        'id': id,
        'name': name,
        'version': version,
        'enabled': enabled,
        'capabilities': capabilities.map((c) => c.wire).toList(),
        'granted': granted.map((c) => c.wire).toList(),
      };

  factory PluginManifestInfo.fromJson(Map<String, dynamic> json) {
    List<PluginCapability> parseCaps(dynamic raw) {
      if (raw is! List) return const [];
      return raw
          .map((e) => PluginCapabilityWire.parse(e.toString()))
          .whereType<PluginCapability>()
          .toList();
    }

    return PluginManifestInfo(
      id: json['id'] as String? ?? '',
      name: json['name'] as String? ?? '',
      version: json['version'] as String? ?? '',
      capabilities: parseCaps(json['capabilities']),
      granted: parseCaps(json['granted']),
      enabled: json['enabled'] as bool? ?? true,
    );
  }
}

/// In-app plugin registry with capability gates (Flutter host mirror of F26.S3).
class PluginRegistry {
  final Map<String, PluginManifestInfo> _plugins = {};
  String? lastError;

  int get pluginCount => _plugins.length;

  List<PluginManifestInfo> list() {
    final items = _plugins.values.toList()
      ..sort((a, b) => a.id.compareTo(b.id));
    return items;
  }

  String listJson() => jsonEncode(list().map((p) => p.toJson()).toList());

  void install({
    required String id,
    required String name,
    String version = '1.0.0',
    required List<PluginCapability> requested,
    required List<PluginCapability> userGranted,
  }) {
    final granted =
        requested.where(userGranted.contains).toList(growable: false);
    _plugins[id] = PluginManifestInfo(
      id: id,
      name: name,
      version: version,
      capabilities: List.of(requested),
      granted: granted,
      enabled: true,
    );
    lastError = null;
  }

  void installSampleEditPlugin({required bool grantEdit}) {
    install(
      id: 'com.tutuaword.sample.edit',
      name: 'Sample Edit Plugin',
      requested: const [
        PluginCapability.documentRead,
        PluginCapability.documentEdit,
      ],
      userGranted: grantEdit
          ? const [
              PluginCapability.documentRead,
              PluginCapability.documentEdit,
            ]
          : const [PluginCapability.documentRead],
    );
  }

  bool enable(String id) {
    final p = _plugins[id];
    if (p == null) {
      lastError = 'unknown plugin: $id';
      return false;
    }
    _plugins[id] = PluginManifestInfo(
      id: p.id,
      name: p.name,
      version: p.version,
      capabilities: p.capabilities,
      granted: p.granted,
      enabled: true,
    );
    lastError = null;
    return true;
  }

  bool disable(String id) {
    final p = _plugins[id];
    if (p == null) {
      lastError = 'unknown plugin: $id';
      return false;
    }
    _plugins[id] = PluginManifestInfo(
      id: p.id,
      name: p.name,
      version: p.version,
      capabilities: p.capabilities,
      granted: p.granted,
      enabled: false,
    );
    lastError = null;
    return true;
  }

  bool uninstall(String id) {
    if (_plugins.remove(id) == null) {
      lastError = 'unknown plugin: $id';
      return false;
    }
    lastError = null;
    return true;
  }

  /// Invoke a plugin command with capability enforcement.
  ///
  /// Returns inserted text for edit plugins, or an error string starting with `ERR:`.
  String invoke(String id, {String documentText = ''}) {
    final p = _plugins[id];
    if (p == null) {
      lastError = 'unknown plugin: $id';
      return 'ERR: unknown plugin';
    }
    if (!p.enabled) {
      lastError = 'plugin disabled: $id';
      return 'ERR: plugin disabled';
    }
    if (!p.hasCapability(PluginCapability.documentEdit)) {
      lastError = 'capability denied: document.edit';
      return 'ERR: capability denied: document.edit';
    }
    lastError = null;
    return 'Hello from plugin$documentText';
  }
}
