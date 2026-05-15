ALTER TABLE review_history
    MODIFY COLUMN mastery_level ENUM('new', 'learning', 'reviewing', 'mastered') NOT NULL