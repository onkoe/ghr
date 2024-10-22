CREATE TABLE reports (
    id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    recv_time TIMESTAMPTZ NOT NULL,
    report JSONB NOT NULL
);