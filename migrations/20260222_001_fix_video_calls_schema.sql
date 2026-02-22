-- Ensure receiver_id is nullable (supports group/multi-party calls where there is no single receiver)
ALTER TABLE video_calls ALTER COLUMN receiver_id DROP NOT NULL;

-- Ensure call_type column exists (added by 20251210 migration, but guard against missing it)
ALTER TABLE video_calls ADD COLUMN IF NOT EXISTS call_type VARCHAR(20) NOT NULL DEFAULT 'video';

-- Ensure group_id column exists
ALTER TABLE video_calls ADD COLUMN IF NOT EXISTS group_id UUID REFERENCES groups(id) ON DELETE SET NULL;

-- Remove the self-call check constraint if it blocks group calls (caller_id = receiver_id is fine for group calls where receiver_id is NULL)
ALTER TABLE video_calls DROP CONSTRAINT IF EXISTS video_calls_check;

-- Ensure call_participants table exists
CREATE TABLE IF NOT EXISTS call_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    call_id UUID NOT NULL REFERENCES video_calls(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) DEFAULT 'participant', -- 'caller', 'participant'
    status VARCHAR(20) DEFAULT 'joined',    -- 'joined', 'left', 'ringing', 'rejected'
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    left_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(call_id, user_id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_call_participants_call_id ON call_participants(call_id);
CREATE INDEX IF NOT EXISTS idx_call_participants_user_id ON call_participants(user_id);
CREATE INDEX IF NOT EXISTS idx_video_calls_group_id ON video_calls(group_id);
