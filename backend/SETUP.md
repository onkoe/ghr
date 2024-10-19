# Setting Up PostgreSQL

Here are the steps I used when getting Postgres set up:

1. On Fedora, install Postgres with `sudo dnf install postgresql-server postgresql-contrib -y`.
1. Generate its configuration: `sudo postgresql-setup --initdb --unit postgresql`.
1. Edit the configuration at `sudo nano /var/lib/pgsql/data/pg_hba.conf`. For IPv4 and IPv6 local connections, change their auth method to `trust` (ONLY IN DEVELOPMENT ENVIRONMENTS) and `md5` with password support.
1. Start the system service with `sudo systemctl start postgresql`.
1. Open up a `psql` with the `postgres` user:
    - In your shell, type: `sudo -u postgres psql`. You're now in `psql`...
    - `CREATE USER barrett WITH SUPERUSER CREATEDB;`
    - `CREATE DATABASE ghr_backend OWNER barrett;`
    - Press Ctrl^D to exit the program.
1. Now, you can be your own user in the new database!
    - In your shell, type: `psql -s barrett -d ghr_backend`.
    - `CREATE TABLE reports (recv_time TIMESTAMPTZ NOT NULL, report JSONB NOT NULL);`
    - Ctrl^D to exit.
1. Compile the `backend` crate. If you get errors, retrace your steps and ensure everything is working as expected.
