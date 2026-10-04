import { Container, getContainer } from "@cloudflare/containers";
import { env } from "cloudflare:workers";

interface Bindings {
  YBM_BACKEND: DurableObjectNamespace<YbmBackend>;
  DATABASE_URL: string;
  REDIS_URL: string;
  ACCESS_TOKEN_SECRET: string;
  REFRESH_TOKEN_SECRET: string;
  R2_ACCESS_KEY_ID: string;
  R2_SECRET_ACCESS_KEY: string;
  R2_BUCKET_NAME: string;
  R2_ENDPOINT: string;
  FCM_CREDENTIALS_JSON: string;
}

const runtimeEnv = env as unknown as Bindings;

export class YbmBackend extends Container<Bindings> {
  defaultPort = 8080;
  sleepAfter = "10m";
  enableInternet = true;
  allowedHosts = [
    "*.supabase.co",
    "*.upstash.io",
    "*.r2.cloudflarestorage.com",
    "oauth2.googleapis.com",
    "fcm.googleapis.com",
    "*.push.services.mozilla.com",
    "*.notify.windows.com",
    "*.push.apple.com",
    "web.push.apple.com"
  ];
  pingEndpoint = "health";
  envVars = {
    APP_ENV: "production",
    SERVER_HOST: "0.0.0.0",
    SERVER_PORT: "8080",
    RUST_LOG: "ybm_connect=info",
    DATABASE_URL: runtimeEnv.DATABASE_URL,
    REDIS_URL: runtimeEnv.REDIS_URL,
    ACCESS_TOKEN_SECRET: runtimeEnv.ACCESS_TOKEN_SECRET,
    REFRESH_TOKEN_SECRET: runtimeEnv.REFRESH_TOKEN_SECRET,
    R2_ACCESS_KEY_ID: runtimeEnv.R2_ACCESS_KEY_ID,
    R2_SECRET_ACCESS_KEY: runtimeEnv.R2_SECRET_ACCESS_KEY,
    R2_BUCKET_NAME: runtimeEnv.R2_BUCKET_NAME,
    R2_ENDPOINT: runtimeEnv.R2_ENDPOINT,
    CORS_ALLOWED_ORIGINS: "",
    EMAIL_PROVIDER: "console",
    EMAIL_FROM: "noreply@localhost",
    PUSH_PROVIDER: "fcm",
    FCM_CREDENTIALS_JSON: runtimeEnv.FCM_CREDENTIALS_JSON
  };
}

export default {
  async fetch(request: Request, workerEnv: Bindings): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname === "/") {
      return new Response("YBM Connect API", { status: 200 });
    }
    return getContainer(workerEnv.YBM_BACKEND, "ybm-api").fetch(request);
  }
};
