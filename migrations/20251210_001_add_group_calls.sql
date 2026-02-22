-- Add call_type and group_id to video_calls, and create call_participants table
ALTER TABLE video_calls ADD COLUMN IF NOT EXISTS call_type VARCHAR(20) DEFAULT 'video';
ALTER TABLE video_calls ADD COLUMN IF NOT EXISTS group_id UUID REFERENCES groups(id) ON DELETE SET NULL;
ALTER TABLE video_calls ALTER COLUMN receiver_id DROP NOT NULL;

CREATE TABLE IF NOT EXISTS call_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    call_id UUID NOT NULL REFERENCES video_calls(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) DEFAULT 'participant', -- 'caller', 'participant'
    status VARCHAR(20) DEFAULT 'joined', -- 'joined', 'left', 'ringing', 'rejected'
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    left_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(call_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_call_participants_call_id ON call_participants(call_id);
CREATE INDEX IF NOT EXISTS idx_call_participants_user_id ON call_participants(user_id);
