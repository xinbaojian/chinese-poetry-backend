-- Update mastery_level ENUM from ('proficient','fair','weak') to ('new','learning','reviewing','mastered')
-- MySQL ALTER ENUM requires re-specifying the full column definition

ALTER TABLE learning_records
    MODIFY COLUMN mastery_level ENUM('new', 'learning', 'reviewing', 'mastered') NOT NULL DEFAULT 'new'
