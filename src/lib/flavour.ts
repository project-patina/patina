/** Set at build time by `scripts/install-dev.sh`; the backend has its own `PATINA_DEV`. */
export const PATINA_DEV = import.meta.env.VITE_PATINA_DEV === "1";
export const APP_NAME = PATINA_DEV ? "Patina Dev" : "Patina";
