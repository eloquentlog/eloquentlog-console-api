Eloquentlog Backend API
=======================

The backend API server of Eloquentlog_.


Requirements
------------

* Rust
* PostgreSQL



Setup
-----

.. code:: zsh

    # set env variables for {production|testing|development}
    % cp .env.sample .env



Build
-----

.. code:: zsh

    % cargo build --release


Development
~~~~~~~~~~~

Use cargo-watch_

.. code:: zsh

    % cargo watch -x 'run' -d 0.3


Testing
~~~~~~~

.. code:: zsh

    % cargo test



License
-------

.. code:: text

   Eloquentlog Backend
   Copyright (c) 2018-2019 Lupine Software LLC


`AGPL-3.0-or-later`


.. code:: text

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU Affero General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU Affero General Public License for more details.

   You should have received a copy of the GNU Affero General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.


.. _Eloquentlog: https://eloquentlog.com/
.. _cargo-watch: https://github.com/passcod/cargo-watch
