-- Fresh deployments start empty; never seed demonstration content publicly.
CREATE TABLE IF NOT EXISTS public.settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
INSERT INTO public.settings(key,value) VALUES('square_seed_disabled','true') ON CONFLICT(key) DO NOTHING;
