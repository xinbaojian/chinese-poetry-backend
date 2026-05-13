CREATE TABLE IF NOT EXISTS poems (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    poet_id BIGINT UNSIGNED NOT NULL,
    dynasty VARCHAR(32) NOT NULL DEFAULT '',
    category VARCHAR(64) NOT NULL DEFAULT '',
    grade TINYINT UNSIGNED NOT NULL DEFAULT 0,
    content JSON NOT NULL,
    translation TEXT,
    source_id VARCHAR(128),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_category (category),
    INDEX idx_grade (grade),
    INDEX idx_dynasty (dynasty),
    INDEX idx_source_id (source_id),
    FOREIGN KEY (poet_id) REFERENCES poets(id)
);
