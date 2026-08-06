import 'dart:async';

typedef AutosaveTick = Future<void> Function();

/// Periodic autosave timer with injectable scheduling for tests.
class AutosaveScheduler {
  AutosaveScheduler({
    required Duration interval,
    required AutosaveTick onTick,
    void Function(Duration interval, void Function() callback)? schedule,
  })  : _interval = interval,
        _onTick = onTick,
        _schedule = schedule ?? _defaultSchedule;

  Duration _interval;
  final AutosaveTick _onTick;
  final void Function(Duration interval, void Function() callback) _schedule;
  Timer? _timer;
  bool _running = false;

  Duration get interval => _interval;

  void start() {
    if (_running) return;
    _running = true;
    _arm();
  }

  void stop() {
    _running = false;
    _timer?.cancel();
    _timer = null;
  }

  void setInterval(Duration interval) {
    _interval = interval;
    if (_running) {
      _timer?.cancel();
      _arm();
    }
  }

  /// Manually fire one autosave tick (tests).
  Future<void> tickNow() => _onTick();

  void _arm() {
    _schedule(_interval, () {
      unawaited(_onTick());
      if (_running) {
        _arm();
      }
    });
  }

  static void _defaultSchedule(Duration interval, void Function() callback) {
    Timer(interval, callback);
  }
}
