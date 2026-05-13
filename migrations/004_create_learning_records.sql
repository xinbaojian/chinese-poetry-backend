CREATE TABLE IF NOT EXISTS learning_records (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT UNSIGNED NOT NULL,
    poem_id BIGINT UNSIGNED NOT NULL,
    mastery_level ENUM('proficient', 'fair', 'weak') NOT NULL DEFAULT 'weak',
    review_count INT UNSIGNED NOT NULL DEFAULT 0,
    next_review_date DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_user_poem (user_id, poem_id),
    INDEX idx_user_next_review (user_id, next_review_date),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (poem_id) REFERENCES poems(id)
);
