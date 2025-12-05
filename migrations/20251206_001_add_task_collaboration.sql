-- Create task_members table for collaborative tasks
CREATE TABLE IF NOT EXISTS task_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'collaborator',
    added_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    added_by UUID REFERENCES users(id),
    CONSTRAINT unique_task_member UNIQUE (task_id, user_id)
);

-- Create task_activity table for audit log
CREATE TABLE IF NOT EXISTS task_activity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    details JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for task_members performance
CREATE INDEX IF NOT EXISTS idx_task_members_task_id ON task_members(task_id);
CREATE INDEX IF NOT EXISTS idx_task_members_user_id ON task_members(user_id);
CREATE INDEX IF NOT EXISTS idx_task_members_role ON task_members(role);

-- Create indexes for task_activity performance
CREATE INDEX IF NOT EXISTS idx_task_activity_task_id ON task_activity(task_id);
CREATE INDEX IF NOT EXISTS idx_task_activity_user_id ON task_activity(user_id);
CREATE INDEX IF NOT EXISTS idx_task_activity_created_at ON task_activity(created_at DESC);

-- Add constraint to ensure role is valid
ALTER TABLE task_members ADD CONSTRAINT check_task_member_role 
    CHECK (role IN ('owner', 'collaborator'));
