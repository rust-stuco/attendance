CREATE TABLE `attendance` (
    `student` TEXT NOT NULL,
    `week` INTEGER NOT NULL,
    `status` TEXT NOT NULL,
    FOREIGN KEY (`student`) REFERENCES students (id),
    FOREIGN KEY (`week`) REFERENCES weeks (id),
    PRIMARY KEY (`student`, `week`)
);
