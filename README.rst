Eloquentlog Console API
=======================

.. image:: https://gitlab.com/eloquentlog/eloquentlog-console-api/badges/master/pipeline.svg
   :target: https://gitlab.com/eloquentlog/eloquentlog-console-api/commits/master

.. image:: https://gitlab.com/eloquentlog/eloquentlog-console-api/badges/master/coverage.svg
   :target: https://gitlab.com/eloquentlog/eloquentlog-console-api/commits/master

.. code:: text

   Eloquentlog

   ╔═╗┌─┐┌┐┌┌─┐┌─┐┬  ┌─┐  ╔═╗╔═╗╦
   ║  │ ││││└─┐│ ││  ├┤   ╠═╣╠═╝║
   ╚═╝└─┘┘└┘└─┘└─┘┴─┘└─┘  ╩ ╩╩  ╩

The console backend API server of Eloquentlog_.


Repository
----------

https://gitlab.com/eloquentlog/eloquentlog-console-api


Requirements
------------

* PostgreSQL
* Redis


Setup
-----

.. code:: zsh

   # set env variables for {production|testing|development}
   % cp .env.sample .env

   # give SUPERUSER to migration user for `CREATE EXTENSION`
   % make schema:migration:commit


Build
-----

.. code:: zsh

   # debug
   % make build

   # see help for building
   % make help | grep 'build '
   ...


Docker
~~~~~~

Prepare ``.env.production``.

.. code:: zsh

   # server
   % docker build --file Dockerfile \
     --build-arg BINARY=server \
     --tag eloquentlog/eloquentlog-console-api-server:latest .
   % docker run --env_file ./.env.production \
     -it eloquentlog/eloquentlog-console-api-server:latest

   # worker
   % docker build --file Dockerfile \
     --build-arg BINARY=worker \
     --tag eloquentlog/eloquentlog-console-api-worker:latest .
   % docker run --env_file ./.env.production \
     -it eloquentlog/eloquentlog-console-api-worker:latest

Note
^^^^

As a common issue, ``--env_file`` doesn't handle double-quoted string like
``FOO="bar"`` because it's not evaluated via shell.


.. code:: zsh

   # this might be something help if you want to connect to host from guest
   % alias host="ip route show 0.0.0.0/0 | grep -Eo 'via \S+' | awk '{print \$2}'"
   % docker run --add-host=postgres:$(host) --add-host=redis:$(host) \
     --env-file ./.env.production \
     --rm -it eloquentlog/eloquentlog-console-api-server


Development
-----------

Vet
~~~

.. code:: zsh

   # see make help about details
   % make verify

Route
~~~~~

To check current routes, run `make route`.

.. code:: zsh

   # print all routes
   % make route
   ...

   # or build router
   % make build:router
   % ./target/debug/router
   ...

Run
~~~

Use cargo-watch_

.. code:: zsh

   % make watch:server
   % make watch:worker

   % curl \
     -H "Content-Type: application/json" \
     -H "Accept: application/json" \
     -d "{}" \
     -X POST \
     http://localhost:8000/_api/signin

Testing
-------

.. code:: zsh

   % ENV=test make schema:migration:commit

   % cargo test model::namespace::test -- --nocapture

   # or run all
   % make test


Deployment
----------

Build
~~~~~

.. code:: zsh

   # e.g. server
   $ IMAGE_NAME=eloquentlog-console-api-server

   % docker build --file Dockerfile \
     --build-arg BINARY=server \
     --tag ${IMAGE_NAME}:latest .

   # e.g. Cloud Registry on Google Cloud Platform
   # - https://cloud.google.com/container-registry/docs/advanced-authentication
   # - https://github.com/GoogleCloudPlatform/docker-credential-gcr
   % gcloud auth configure-docker

   # or use docker-credential-gcr
   % VERSION=...
   % OS=linux
   % ARCH=amd64
   % curl -fsSL "https://.../v${VERSION}/..._${OS}_${ARCH}-${VERSION}.tar.gz" \
     | tar xz --to-stdout ./docker-credential-gcr \
     > .tool/docker-credential-gcr && \
     chmod +x .tool/docker-credential-gcr
   % .tool/docker-credential-gcr configure-docker

   # push
   % PROJECT_ID=...
   % HOST=eu.gcr.io
   % docker tag ${IMAGE_NAME}:latest ${HOST}/${PROJECT_ID}/${IMAGE_NAME}:latest
   % docker push ${HOST_NAME}/${PROJECT_ID}/${IMAGE_NAME}:latest


Cloud Run
~~~~~~~~~

This is experimental. The following middlewares are required.

* ``Cloud SQL`` for PostgreSQL
* ``Memorystore`` for Redis (with VPC connector)
* ``Cloud Storage`` (for logging)

The build will be done via ``Cloud Build``.

0. Setup gcloud on local
^^^^^^^^^^^^^^^^^^^^^^^^

.. code:: zsh

   % .tool/setup-cloud-sdk
   % source .tool/load-gcloud

1. Prepare env vars for applications
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

The ``server`` and ``worker`` both applications use same dotenv file.

.. code:: zsh

   % cp .env.deploy.sample .env.deploy
   % $EDITOR .env.deploy

There is some note for the postgres connection via unix socket.
``/.s.PGSQL.5432`` will be appended automatically, and slash and colon must be
escaped in DATABASE_URL. So, it should look like:
``DATABASE_URL="postgresql://user:password@%2Fpath%2Fto%2Fdir%3Afoo%3Abar``

2. Prepare env vars for deploy
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Especially ``GCP_CLOUD_BUILD_SUBSTR_ENV_VARS`` must be a path to the file
``.env.deploy`` which has been created in above step.

.. code:: zsh

   % cp .env.ci.sample .env.ci
   % $EDITOR .env.ci

   # each line has export (for make)
   % source .env.ci

3. Run make deploy
^^^^^^^^^^^^^^^^^^

Currently, it may take more than 30 minutes...

.. code:: zsh

   % make deploy:server
   % make deploy:worker


License
-------

.. code:: text

   ┏━╸╻  ┏━┓┏━┓╻ ╻┏━╸┏┓╻╺┳╸╻  ┏━┓┏━╸
   ┣╸ ┃  ┃ ┃┃┓┃┃ ┃┣╸ ┃┗┫ ┃ ┃  ┃ ┃┃╺┓
   ┗━╸┗━╸┗━┛┗┻┛┗━┛┗━╸╹ ╹ ╹ ┗━╸┗━┛┗━┛

   Console API
   Copyright (c) 2018-2019 Lupine Software LLC


``AGPL-3.0-or-later``.

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
