==========
Changelog
==========

All notable changes to this project (especially breaking ones) will be
documented in this file.

This project adheres to `Semantic Versioning <https://semver.org/>`_.

`0.6.0`_ (2020-02-04)
---------------------

Bug Fixes
~~~~~~~~~

* Updated version of ``bindgen`` to allow building on Rust 1.39 and greater.
* Changed treatment of enums from ``tibrv.h`` in ``tibrv-sys``, see `801b6ae <https://github.com/bradfier/tibrv-rs/commit/801b6ae28421da6a9f87d834206dcfeef686df39>`_
  for explanation. This shouldn't be a change visible outside the ``-sys`` crate,
  but may affect you if you use this crate directly. (This is the reasoning for the full
  version bump.)


`0.5.0`_ (2019-07-01)
---------------------

New Features
~~~~~~~~~~~~

* Rendezvous Request / Response (with Async support)

  See `async_client.rs <https://github.com/bradfier/tibrv-rs/blob/master/examples/async_client.rs>`_
  and `async_server.rs <https://github.com/bradfier/tibrv-rs/blob/master/examples/async_server.rs>`_
  for usage examples.

Bug Fixes
~~~~~~~~~

* A number of lifetime issues have been cleaned up in ``Msg`` and ``MsgField``
  thanks to @pfernie
* Async queues no longer leak two words every time they are dropped.

Breaking Changes
~~~~~~~~~~~~~~~~

* Async ``Stream`` and ``Sink`` use ``TibrvError`` to indicate fault conditions rather
  than a blanket ``io::Error``.

`0.4.0`_ (2018-07-18)
---------------------

* Added ``get_field_by_index`` for ``tibrv::Msg``
* Fixed an early-drop bug where ``TransportBuilder`` was used with
  optional parameters, the ``network``, ``service`` or ``daemon``
  strings could be dropped before being handed off to the C library.
* Fixed examples to build with appropriate feature flags enabled.

Also in this release are two **breaking changes**:

* For consistency with the standard library, in ``TibrvResult`` we have
  renamed ``and_then`` to ``map`` and introduced a new ``and_then``
  function which takes a closure returning a ``Result<T, TibrvError>``,
  while ``map`` now accepts a closure returning ``T``.
* Added *feature gating* for different versions of the Rendezvous
  library, see `README.md <https://github.com/bradfier/tibrv-rs/blob/master/README.md>`_
  for more information.

`0.3.0`_ (2018-06-06)
---------------------

* Migrates to Tokio 0.1, with API changes required to accomodate new
  constraints imposed by the Tokio and Futures crates.
  This release contains **breaking changes** which are documented in more
  detail `here <https://fstab.me/posts/tibrv-0.3.0.html>`_.

`0.2.0`_ (2017-12-12)
---------------------

* Introduces proper error handling using the ``Failure`` crate,
  with a ``TibrvError`` type and associated ``ErrorKind`` to easily
  handle error conditions from the underlying Rendezvous library.
  This is a **breaking change**, as most library functions now have a
  return type of ``Result<T, TibrvError>`` instead of ``Result<T, &'static str>``.

`0.1.1`_ (2017-10-30)
---------------------

Initial public release.

* Added sync & async APIs for basic Rendezvous usage, along with convenience
  methods for encoding and decoding message fields.
* First usable release on `crates.io <https://crates.io/crates/tibrv>`_.


.. _`0.6.0`: https://github.com/bradfier/tibrv-rs/compare/v0.5.0...v0.6.0
.. _`0.5.0`: https://github.com/bradfier/tibrv-rs/compare/v0.4.0...v0.5.0
.. _`0.4.0`: https://github.com/bradfier/tibrv-rs/compare/v0.3.0...v0.4.0
.. _`0.3.0`: https://github.com/bradfier/tibrv-rs/compare/v0.2.0...v0.3.0
.. _`0.2.0`: https://github.com/bradfier/tibrv-rs/compare/v0.1.1...v0.2.0
.. _`0.1.1`: https://github.com/bradfier/tibrv-rs/compare/2947f836...v0.1.1
