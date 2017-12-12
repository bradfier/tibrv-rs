==========
Changelog
==========

All notable changes to this project (especially breaking ones) will be
documented in this file.

This project adheres to `Semantic Versioning <https://semver.org/>`_.

Upcoming Changes
----------------

* Rendezvous Request / Reply
* Rendezvous Distributed Queues

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


.. _`0.2.0`: https://github.com/bradfier/tibrv-rs/compare/v0.1.1...v0.2.0
.. _`0.1.1`: https://github.com/bradfier/tibrv-rs/compare/2947f836...v0.1.1
