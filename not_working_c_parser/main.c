#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <ctype.h>
#include <regex.h>

int pc = 0;

typedef struct {
    char name[1024];
    int index;
} label;

label labels[1024];


void replace_all(char *str, const char *what, const char *with) {
    char buffer[1024] = {0};
    char *insert_point = &buffer[0];
    const char *tmp = str;

    size_t what_len = strlen(what);
    size_t with_len = strlen(with);

    while (1) {
        const char *p = strstr(tmp, what);

        if (p) {
            memcpy(insert_point, tmp, p - tmp);
            insert_point += p - tmp;

            memcpy(insert_point, with, with_len);
            insert_point += with_len;

            tmp = p + what_len;
        } else {
            strcpy(insert_point, tmp);
            break;
        }
    }

    strcpy(str, buffer);
}

uint16_t parserRegister(char *s) {
    if (s != NULL && s[0] == 'r') {
        char *substr = &s[1];
        int value = atoi(substr);
        if (value >= 0 && value < 8) {
            printf("register: %b\n", value);
            return (uint16_t) value;
        }
    }
    fprintf(stderr, "register not parsed\n");
    exit(1);
}

uint16_t parseImm(int T, char *s, int sp) {
    if (s != NULL && s[0] == '#') {
        char *substr = &s[1];
        int value = atoi(substr) / (sp ? 4 : 1);
        if (value >= 0 && value < (1 << T)) {
            printf("register: %b\n", value);
            return (uint16_t) value;
        }
    }

    fprintf(stderr, "immediate not parsed\n");
    exit(EXIT_FAILURE);
}

uint16_t parseCondition(char *s) {
    for (int i = 0; s[i]; i++) {
        s[i] = toupper((unsigned char) s[i]);
    }

    if (strcmp(s, "EQ") == 0)
        return 0b0000000000000000;
    if (strcmp(s, "NE") == 0)
        return 0b0000000000000001;
    if (strcmp(s, "CS") == 0)
        return 0b0000000000000010;
    if (strcmp(s, "CC") == 0 || strcmp(s, "LO") == 0)
        return 0b0000000000000011;
    if (strcmp(s, "MI") == 0)
        return 0b0000000000000100;
    if (strcmp(s, "PL") == 0)
        return 0b0000000000000101;
    if (strcmp(s, "VS") == 0)
        return 0b0000000000000110;
    if (strcmp(s, "VC") == 0)
        return 0b0000000000000111;
    if (strcmp(s, "HI") == 0)
        return 0b0000000000001000;
    if (strcmp(s, "LS") == 0)
        return 0b0000000000001001;
    if (strcmp(s, "GE") == 0)
        return 0b0000000000001010;
    if (strcmp(s, "LT") == 0)
        return 0b0000000000001011;
    if (strcmp(s, "GT") == 0)
        return 0b0000000000001100;
    if (strcmp(s, "LE") == 0)
        return 0b0000000000001101;
    if (strcmp(s, "AL") == 0)
        return 0b0000000000001110;

    fprintf(stderr, "condition not recognized: %s\n", s);
    exit(EXIT_FAILURE);
}

void binprintf(int v) {
    unsigned int mask = 1 << ((sizeof(int) << 3) - 1);
    while (mask) {
        printf("%d", (v & mask ? 1 : 0));
        mask >>= 1;
    }
    printf("\n");
}

int parseBranchJump(char *label) {
    for (int i = 0; i < 1024; i++) {
        if (strcmp(labels[i].name, label) == 0) {
            return labels[i].index;
        }
    }
    fprintf(stderr, "label not found: %s\n", label);
    exit(EXIT_FAILURE);
}

uint16_t parseLabel(int T, const char *s, int source) {
    int labelIndex = parseBranchJump(s);
    printf("labelIndex: %d\n", labelIndex);
    int offset = (labelIndex - source) - 4;
    if (offset >= -(1 << (T - 1)) && offset < (1 << (T - 1))) {
        return (uint16_t) (offset & ((1 << T) - 1));
    }
    fprintf(stderr, "label not parsed\n");
    exit(EXIT_FAILURE);
}

int execute_regex(char *regex, char *instruction) {
    regex_t reg;
    regcomp(&reg, regex, REG_EXTENDED);
    int status = regexec(&reg, instruction, 0, NULL, 0);
    regfree(&reg);
    return status == 0;
}

uint16_t convert_instruction(char *instruction, char **args, int args_size) {
    if (execute_regex("^lsls?$", instruction)) {
        if (args_size == 3) {
            return 0b0000000000000000 |
                   parserRegister(args[2]) << 6 |
                   parserRegister(args[1]) << 3 |
                   parserRegister(args[0]);
        }
        if (args_size == 2) {
            return 0b0100000010000000 |
                   parserRegister(args[1]) << 3 |
                   parserRegister(args[0]);
        }
    } else if (execute_regex("^lsrs?$", instruction)) {
        if (args_size == 3) {
            return 0b0000100010000000 |
                   parserRegister(args[2]) << 6 |
                   parserRegister(args[1]) << 3 |
                   parserRegister(args[0]);
        }
        if (args_size == 2) {
            return 0b0100000011000000 |
                   parserRegister(args[1]) << 3 |
                   parserRegister(args[0]);
        }
    } else if (execute_regex("^asrs?$", instruction)) {
        if (args_size == 3) {
            return 0b0001000000000000 |
                   parserRegister(args[2]) << 6 |
                   parserRegister(args[1]) << 3 |
                   parserRegister(args[0]);
        }
    } else if (execute_regex("^adds?$", instruction)) {
        printf("ADD DETECTED\n");
        for (int i = 0; i < args_size; i++) {
            printf("arg %s\n", args[i]);
        }
        if (strcmp(args[0], "sp") != 0 && strcmp(args[1], "sp") != 0) {
            if (args_size == 3 && args[2][0] == 'r') {
                return 0b0001100000000000 |
                       parserRegister(args[2]) << 6 |
                       parserRegister(args[1]) << 3|
                       parserRegister(args[0]);
            }
            if (args_size == 3 && args[2][0] == '#') {
                return 0b0001110000000000 |
                        parseImm(3, args[2], 0) << 6 |
                        parseImm(3, args[1], 0) << 3 |
                        parseImm(3, args[0], 0);
                /*
                       parserRegister(args[0]) << 8 |
                       parseImm(8, args[args_size - 1], 0);
*/
            }
            return 0b0001100000000000 |

                    parserRegister(args[0]) << 8 |
                    parseImm(8, args[args_size - 1], 0);

                    /*
                   parserRegister(args[2]) << 6 |
                   parserRegister(args[1]) << 3 |
                   parserRegister(args[0]);*/

        }
        if (strcmp(args[0], "sp") == 0) {
            return 0b1011000000000000 |
                   parseImm(7, args[args_size-1], 1);
        }
        // if rd sp imm8
        return 0b1011000010000000 |
               parseImm(7, args[2], 1);
    } else if (execute_regex("^subs?$", instruction)) {
        if (args_size == 3) {
            if (args[2][0] == '#') {
                return 0b0001111000000000 |
                       parseImm(8, args[2], 0) << 6 |
                       parserRegister(args[1]) << 3 |
                       parserRegister(args[0]);
            }
            return 0b0001101000000000 |
                   parserRegister(args[2]) << 6 |
                   parserRegister(args[1]) << 3 |
                   parserRegister(args[0]);
        }
        if (args_size == 2) {
            if (strcmp(args[0], "sp") != 0) {
                return 0b0011100000000000 |
                       parserRegister(args[0]) << 8 |
                       parseImm(8, args[1], 0);
            }
            return 0b1011000010000000 |
                   parseImm(7, args[1], 1);
        }
    } else if (execute_regex("^movs?$", instruction)) {
        if (args_size == 2) {
            if (args[1][0] == '#') {
                printf("movs %s\n", args[1]);
                // print in binary
                binprintf(parseImm(8, args[1], 0));
                binprintf((parserRegister(args[0]) << 8));
                return (uint16_t) 0b0010000000000000 |
                       parserRegister(args[0]) << 8 |
                       parseImm(8, args[1], 0);
            }
            else{
                return (uint16_t) 0b0000000000000000 |
                       parserRegister(args[1]) << 3 |
                       parserRegister(args[0]);
            }
        }
        
    } else if (execute_regex("^cmp$", instruction)) {
        if (args[1][0] == '#') {
            return 0b0010100000000000 |
                   parserRegister(args[0]) << 8 |
                   parseImm(8, args[1], 0);
        }
        return 0b0100001010000000 |
               parserRegister(args[1]) << 3 |
               parserRegister(args[0]);
    } else if (execute_regex("^ands?$", instruction)) {
        if (args_size == 2) {
            return 0b0100000000000000 |
                   parserRegister(args[1]) << 3 |
                   parserRegister(args[0]);
        }
    } else if (execute_regex("^eors?$", instruction)) {
        if (args_size == 2) {
            return (uint16_t) (0b0100000001000000) |
                   parserRegister(args[1]) << 3 | parserRegister(args[0]);
        }
    } else if (execute_regex("^asrs?$", instruction)) {
        if (args_size == 2) {
            return (uint16_t) (0b01000000100000000) |
                   parserRegister(args[1]) << 3 | parserRegister(args[0]);
        }
    } else if (execute_regex("^adcs?$", instruction)) {
        if (args_size == 2) {
            return (uint16_t) (0b0100000101000000) |
                   parserRegister(args[1]) << 3 | parserRegister(args[0]);
        }
    } else if (execute_regex("^sbcs?$", instruction)) {
        if (args_size == 2) {
            return (uint16_t) (0b0100000110000000) |
                   parserRegister(args[1]) << 3 | parserRegister(args[0]);
        }
    } else if (execute_regex("^rors$", instruction)) {
        if (args_size == 2) {
            return (uint16_t) (0b0100000111000000) |
                   parserRegister(args[1]) << 3 | parserRegister(args[0]);
        }
    } else if (execute_regex("^tst$", instruction)) {
        if (args_size == 2) {
            return (uint16_t) (0b0100001000000000) |
                   parserRegister(args[1]) << 3 | parserRegister(args[0]);
        }
    } else if (execute_regex("^rsbs?$", instruction)) {
        if (args_size == 3) {
            return (uint16_t) (0b0100001001000000) |
                   parserRegister(args[1]) << 3 | parserRegister(args[0]);
        }
    } else if (execute_regex("^cmn$", instruction)) {
        if (args_size == 2) {
            return (uint16_t) (0b0100001011000000) |
                   parserRegister(args[1]) << 3 | parserRegister(args[0]);
        }
    } else if (execute_regex("^orrs?$", instruction)) {
        if (args_size == 2) {
            return (uint16_t) (0b0100001100000000) |
                   parserRegister(args[1]) << 3 | parserRegister(args[0]);
        }
    } else if (execute_regex("^muls?$", instruction)) {
        if (args_size == 3) {
            return (uint16_t) (0b0100001101000000) |
                   parserRegister(args[1]) << 3 | parserRegister(args[0]);
        }
    } else if (execute_regex("^bics?$", instruction)) {
        if (args_size == 2) {
            return (uint16_t) (0b0100001110000000) |
                   parserRegister(args[1]) << 3 | parserRegister(args[0]);
        }
    } else if (execute_regex("^mvns?$", instruction)) {
        if (args_size == 2) {
            return (uint16_t) (0b0100001111000000) |
                   parserRegister(args[1]) << 3 | parserRegister(args[0]);
        }
    } else if (execute_regex("^str$", instruction)) {
        uint16_t bin = (0b1001000000000000);
        if (args_size >= 3) {
            return bin | parserRegister(args[0]) << 8 |
                   (uint16_t) (parseImm(8, args[2], 0) / 4);
        }
        return bin | parserRegister(args[0]) << 8;
    } else if (execute_regex("^ldr$", instruction)) {
        if (args_size >= 3) {
            return (uint16_t) (0b1001100000000000) |
                   parserRegister(args[0]) << 8 |
                   (uint16_t) (parseImm(16, args[2], 0) / 4);
        }
        return (uint16_t) (0b1001100000000000) |
               parserRegister(args[0]) << 8;
    } else if (execute_regex("^cmp?$", instruction)) {
        if (args[1][0] == '#') {
            return (uint16_t) (0b0010100000000000) |
                   parserRegister(args[0]) << 8 | parseImm(8, args[1], 0);
        }
        return (uint16_t) (0b0100001010000000) |
               parserRegister(args[1]) << 3 | parserRegister(args[0]);
    } else if (execute_regex("^b.{0,2}$", instruction)) {
        if (strcmp(instruction, "b") == 0 || strcmp(instruction, "bx") == 0) {
            printf("arg %s\n", args[0]);
            binprintf(parseLabel(11, args[0], pc));
            return (uint16_t) (0b1110000000000000 | parseLabel(11, args[0], pc));
        }
        return 0b1101000000000000 | parseCondition(instruction + 1) << 8 | parseLabel(8, args[0], pc);
    } else if (execute_regex("^push$", instruction)) {
        printf("Skipping push\n");
        return 0b0000000000000000;
    }
    fprintf(stderr, "instruction not recognized: %s\n", instruction);
    exit(EXIT_FAILURE);
}


void replace_extension(char *in_file, char *str, char *bin) {
    char *dot = strrchr(in_file, '.');
    if (!dot || dot == in_file) {
        strcpy(str, in_file);
        strcat(str, bin);
    } else {
        size_t len = dot - in_file;
        strncpy(str, in_file, len);
        str[len] = '\0';
        strcat(str, bin);
    }
}

void trim(char *str) {
    // remove spaces and tabs from beginning and end of string
    int i;
    int begin = 0;
    int end = strlen(str) - 1;

    while (isspace((unsigned char) str[begin]))
        begin++;

    while ((end >= begin) && isspace((unsigned char) str[end]))
        end--;

    // Shift all characters back to the start of the string array.
    for (i = begin; i <= end; i++)
        str[i - begin] = str[i];

    str[i - begin] = '\0'; // Null terminate string.
}

int main(int argc, char **argv) {
    char *labelRegex = ("^.*:$");
    char *instructionRegex = (
            "^[add|sub|mov|cmp|and|eor|lsl|lsr|asr|adc|sbc|ror|tst|rsb|cmn|orr|mul|bic|mvn|str|ldr|push|b|bx|beq|bne|bcs|bcc|bmi|bpl|bvs|bvc|bhi|bls|bge|blt|bgt|ble].*$");
    if (argc > 1) {
        char *inFile = argv[1];
        char *extensionRegex = ("^(.*)[.]s$");

        if (execute_regex(extensionRegex, inFile)) {
            char outFile[1024];
            replace_extension(inFile, outFile, ".bin");

            FILE *in = fopen(inFile, "r");
            FILE *out = fopen(outFile, "w");

            int ol = 0;
            if (in) {
                printf("assembling in file %s\n", outFile);

                char line[1024];
                while (fgets(line, sizeof(line), in)) {
                    trim(line);
                    if (execute_regex(labelRegex, line) && !execute_regex("run:", line)) {
                        char *label = strtok(line, ":");
                        labels[ol].index = pc;
                        strcpy(labels[ol].name, label);
                        printf("label: %s(%d)\n", labels[ol].name, labels[ol].index);
                    } else if (execute_regex(instructionRegex, line)) {
                        ++pc;
                    }
                    ol++;
                }


                pc = 0;

                rewind(in);

                while (fgetc(out) != EOF) {
                    fputc(0, out);
                }

                fprintf(out, "v2.0 raw\n");

                while (fgets(line, sizeof(line), in)) {
                    replace_all(line, ", ", ",");
                    replace_all(line, "[", "");
                    replace_all(line, "]", "");
                    trim(line);
                    if (execute_regex(instructionRegex, line) && !execute_regex("run:", line)) {
                        char *instruction = strtok(line, "\t");
                        char *args[3];
                        int i = 0;

                        while ((args[i] = strtok(NULL, ",")) != NULL) {
                            i++;
                        }
                        fprintf(out, "%04x ", convert_instruction(instruction, args, i));

                        printf("%s(", instruction);
                        for (int j = 0; j < i; j++) {
                            printf("%s,", args[j]);
                        }
                        printf(")\n");

                        ++pc;
                    }
                }

                fprintf(out, "\n");
                printf("avengers assembled in %s\n", outFile);
            } else {
                printf("file %s being non-existent.\n", inFile);
                return 1;
            }

            fclose(in);
            fclose(out);
        } else {
            printf("file %s not being in assembly format (*.s).\n", inFile);
            return 1;
        }
    } else {
        printf("lack of argument.\n");
        return 1;
    }
    return 0;
}