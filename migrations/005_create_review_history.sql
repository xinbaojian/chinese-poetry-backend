CREATE TABLE IF NOT EXISTS review_history (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    learning_record_id BIGINT UNSIGNED NOT NULL,
    reviewed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    mastery_level ENUM('proficient', 'fair', 'weak') NOT NULL,
    FOREIGN KEY (learning_record_id) REFERENCES learning_records(id) ON DELETE CASCADE
);
