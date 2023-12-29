#define FILE char // TODO should be `typedef`

#define stdin 0  // TODO should be `inline const FILE *stdin;`
#define stdout 0 // TODO should be `inline const FILE *stdout;`
#define stderr 0 // TODO should be `inline const FILE *stdout;`

#define NULL 0
#define EOF -1

// TODO shoud be `int fgetc(FILE*);`
inline char fgetc(FILE *stream);
// TODO shoud be `int getchar(void);`
inline char getc(void);
// TODO shoud be `char* fgets(char* buf, int size, FILE* stream);`
void fgets(FILE *stream, char *buf);
// TODO should be `char* gets(char* buf);`
void gets(char *buf);

// TODO should be `int fputc(int c, FILE* stream);`
inline void fputc(FILE *stream, char c);
// TODO should be `int putchar(int c);`
inline void putc(char c);
// TODO should be `int fputs(const char* buf, FILE* stream);`
void fputs(FILE *stream, const char *buf);
// TODO should be `int puts(const char* buf);`
void puts(const char *buf);

void printf(char *format, ...); // TODO should return `int`
