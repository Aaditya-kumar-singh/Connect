declare global {
  interface Env {
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
}
export {};