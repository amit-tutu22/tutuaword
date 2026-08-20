import 'dart:ffi';
import 'dart:typed_data';

void Function(Pointer<Uint8>, int)? _freeBufferFn;
final _bufferLengths = <int, int>{};
final _ffiBufferFinalizer = Finalizer<Pointer<Uint8>>((ptr) {
  final len = _bufferLengths.remove(ptr.address);
  final free = _freeBufferFn;
  if (len != null && free != null) {
    free(ptr, len);
  }
});

/// Bind the Dart wrapper around native `tw_free_buffer` once per process.
void bindFfiBufferFinalizer(void Function(Pointer<Uint8>, int) freeBuffer) {
  _freeBufferFn = freeBuffer;
}

/// Adopt an FFI-owned byte buffer without copying into the Dart heap.
Uint8List adoptFfiBuffer(Pointer<Uint8> ptr, int len) {
  if (ptr == nullptr || len == 0) {
    return Uint8List(0);
  }
  if (_freeBufferFn == null) {
    throw StateError('bindFfiBufferFinalizer must be called before adoptFfiBuffer');
  }
  final view = ptr.asTypedList(len);
  _bufferLengths[ptr.address] = len;
  _ffiBufferFinalizer.attach(view, ptr);
  return view;
}
