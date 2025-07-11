-- SQL script to create an admin user directly in the database
-- This bypasses the API and creates the admin user with all necessary permissions

-- Generate a UUID for the admin user
INSERT INTO users (
    id,
    username,
    email,
    password_hash,
    is_active,
    role,
    is_admin,
    permissions,
    created_at,
    updated_at
) VALUES (
    gen_random_uuid(),
    'admin',
    'admin@example.com',
    '$2b$12$LQv3c1yqBWVHxkd0LQ4YCOydbZdnNgc4rKWKvTDznYaEON3Yiq3Y2', -- Password: "admin123"
    true,
    'admin',
    true,
    ARRAY[
        'system:*',
        'users:*',
        'projects:*',
        'translations:*',
        'api_keys:*'
    ],
    NOW(),
    NOW()
) ON CONFLICT (username) DO UPDATE SET
    role = 'admin',
    is_admin = true,
    permissions = ARRAY[
        'system:*',
        'users:*',
        'projects:*',
        'translations:*',
        'api_keys:*'
    ],
    updated_at = NOW();

-- Verify the admin user was created
SELECT id, username, email, role, is_admin, permissions FROM users WHERE username = 'admin';