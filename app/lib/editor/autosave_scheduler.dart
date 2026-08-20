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
        _schedule = schedule;

  Duration _interval;
  final AutosaveTick _onTick;
  final void Function(Duration interval, void Function() callback)? _schedule;
  Timer? _timer;
  bool _running = false;
  int _generation = 0;

  Duration get interval => _interval;

  void start() {
    if (_running) return;
    _running = true;
    _arm();
  }

  void stop() {
    _running = false;
    _generation++;
    _timer?.cancel();
    _timer = null;
  }

  void setInterval(Duration interval) {
    _interval = interval;
    if (_running) {
      _generation++;
      _timer?.cancel();
      _timer = null;
      _arm();
    }
  }

  /// Manually fire one autosave tick (tests).
  Future<void> tickNow() => _onTick();

  void _arm() {
    final gen = ++_generation;
    if (_schedule != null) {
      _schedule(_interval, () {
        unawaited(_onTickCompleted(gen));
      });
      return;
    }
    _timer?.cancel();
    _timer = Timer(_interval, () {
      unawaited(_onTickCompleted(gen));
    });
  }

  Future<void> _onTickCompleted(int gen) async {
    if (!_running || gen != _generation) return;
    await _onTick();
    if (_running && gen == _generation) {
      _arm();
    }
  }
}
