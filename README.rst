Eloquentlog Backend API
=======================

.. image:: https://gitlab.com/eloquentlog/eloquentlog-backend-api/badges/master/pipeline.svg
   :target: https://gitlab.com/eloquentlog/eloquentlog-backend-api/commits/master

.. image:: https://gitlab.com/eloquentlog/eloquentlog-backend-api/badges/master/coverage.svg
   :target: https://gitlab.com/eloquentlog/eloquentlog-backend-api/commits/master

.. code:: text

   Eloquentlog

   ╔╗ ┌─┐┌─┐┬┌─┌─┐┌┐┌┌┬┐  ╔═╗╔═╗╦
   ╠╩╗├─┤│  ├┴┐├┤ │││ ││  ╠═╣╠═╝║
   ╚═╝┴ ┴└─┘┴ ┴└─┘┘└┘─┴┘  ╩ ╩╩  ╩

The backend API server of Eloquentlog_.


Repository
----------

https://gitlab.com/eloquentlog/eloquentlog-backend-api


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
      --tag eloquentlog/eloquentlog-backend-api-server:latest .
    % docker run --env_file ./.env.production \
      -it eloquentlog/eloquentlog-backend-api-server:latest

    # worker
    % docker build --file Dockerfile \
      --build-arg BINARY=worker \
      --tag eloquentlog/eloquentlog-backend-api-worker:latest .
    % docker run --env_file ./.env.production \
      -it eloquentlog/eloquentlog-backend-api-worker:latest

Note
^^^^

As a common issue, ``--env_file`` doesn't handle double-quoted string like
``FOO="bar"`` because it's not evaluated via shell.


.. code:: zsh

    # this might be something help if you want to connect to host from guest
    % alias host="ip route show 0.0.0.0/0 | grep -Eo 'via \S+' | awk '{print \$2}'"
    % docker run --add-host=postgres:$(host) --add-host=redis:$(host) \
      --env-file ./.env.production \
      --rm -it eloquentlog/eloquentlog-backend-api-server


Development
-----------

Vet
~~~

.. code:: zsh

    # see make help about details
    % make verify

Run
~~~

Use cargo-watch_

.. code:: zsh

    % make watch:serve
    % make watch:queue

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

    % make test


Deployment
----------

.. code:: zsh

    # e.g. server
    $ IMAGE_NAME=eloquentlog-backend-api-server

    % docker build --file Dockerfile \
      --build-arg BINARY=server \
      --tag eloquentlog/${IMAGE_NAME}:latest .

    # e.g. publish the image to Cloud Registry on Google Cloud Platform
    # - https://cloud.google.com/container-registry/docs/advanced-authentication
    # - https://github.com/GoogleCloudPlatform/docker-credential-gcr
    % VERSION=...
    % OS=linux
    % ARCH=amd64
    % curl -fsSL "https://.../v${VERSION}/..._${OS}_${ARCH}-${VERSION}.tar.gz" \
      | tar xz --to-stdout ./docker-credential-gcr \
      > .tool/docker-credential-gcr && \
      chmod +x .tool/docker-credential-gcr
    % .tool/docker-credential-gcr configure-docker

    % PROJECT_ID=...
    % HOST_NAME=eu.gcr.io
    % docker push $HOST_NAME/${PROJECT-ID}/${IMAGE_NAME}:latest


License
-------

.. code:: text

   ┏━╸╻  ┏━┓┏━┓╻ ╻┏━╸┏┓╻╺┳╸╻  ┏━┓┏━╸
   ┣╸ ┃  ┃ ┃┃┓┃┃ ┃┣╸ ┃┗┫ ┃ ┃  ┃ ┃┃╺┓
   ┗━╸┗━╸┗━┛┗┻┛┗━┛┗━╸╹ ╹ ╹ ┗━╸┗━┛┗━┛

   Backend API
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
