package shield;

import java.io.BufferedReader;
import java.io.FileReader;
import java.io.FileWriter;
import java.io.IOException;
import java.util.*;

public class Avengers {
    private final List<String> lines = new ArrayList<>();
    private final Map<String, Integer> labels = new HashMap<>();
    private final StringBuilder stringBuilder = new StringBuilder("v2.0 raw\n");
    private int instructionCount = 0;

    public void assemble(String[] args) {
        preParser(args);
        lines.forEach(line -> parser(damageControl(line)));
        writeOutput(stringBuilder.toString(), args);
    }

    private String[] damageControl(String string) {
        String[] temp = string.trim().split("\\s+");
        if (temp[0].endsWith(":") || temp[0].contains(".addrsig")) {
            return new String[0];
        }
        if (temp[0].equals("b") && temp.length > 1) {
            temp[1] = temp[1].trim();
        }
        for (int i = 1; i < temp.length; i++) {
            temp[i] = temp[i].replaceAll(",|\\[sp, |\\[|]", "").trim();
        }
        return temp;
    }

    private void preParser(String[] args) {
        final String[] mnemonic = new String[]{"lsls", "lsrs", "asrs", "add", "sub", "movs", "cmp", "ands", "eors", "adcs", "sbcs", "rors", "tst", "rsbs", "cmn", "orrs", "muls", "bics", "mvns", "str", "ldr", "b\t"};
        int currentLine = 1;
        try (BufferedReader bufferedReader = new BufferedReader(new FileReader(args[0]))) {
            String line;
            while ((line = bufferedReader.readLine()) != null) {
                line = line.trim();
                if (Arrays.stream(mnemonic).anyMatch(line::contains) || line.startsWith("b")) {
                    lines.add(line);
                    currentLine++;
                }
                if (line.endsWith(":")) {
                    labels.put(line, currentLine);
                }
            }
        } catch (IOException e) {
            System.out.println("AVENGERS! DISASSEMBLE! : File not found");
        }
    }

    private void writeOutput(String buffer, String[] args) {
        try (FileWriter writer = new FileWriter(args[0].replace(".s", ".bin"))) {
            writer.write(buffer);
        } catch (IOException e) {
            System.out.println("AVENGERS! DISASSEMBLE! : File not found");
        }
    }

    private void parser(String[] line) {
        if (line.length == 0) return;
        instructionCount++;
        switch (line[0]) {
            case "lsls":
                if (line.length == 4) {
                    lslImm5(line[1], line[2], line[3]);
                } else {
                    lslReg(line[1], line[2]);
                }
                break;
            case "lsrs":
                if (line.length == 4) {
                    lsrImm5(line[1], line[2], line[3]);
                } else {
                    lsrReg(line[1], line[2]);
                }
                break;
            case "asrs":
                if (line.length == 4) {
                    asrImm5(line[1], line[2], line[3]);
                } else {
                    asrReg(line[1], line[2]);
                }
                break;
            case "adds":
                if (line[2].contains("#")) {
                    addImm8(line[1], line[2]);
                } else if (line[3].contains("#")) {
                    if ((intToBinaryString(line[3], 0)).length() <= 3) {
                        addImm3(line[1], line[2], line[3]);
                    } else if (Objects.equals(line[1], line[2])) {
                        addImm8(line[1], line[3]);
                    }
                } else {
                    addReg(line[1], line[2], line[3]);
                }
                break;
            case "subs":
                if (line[2].contains("#")) {
                    subImm8(line[1], line[2]);
                } else if (line[3].contains("#")) {
                    if ((intToBinaryString(line[3], 0)).length() <= 3) {
                        subImm3(line[1], line[2], line[3]);
                    } else if (Objects.equals(line[1], line[2])) {
                        subImm8(line[1], line[3]);
                    }
                } else {
                    subReg(line[1], line[2], line[3]);
                }
                break;
            case "movs":
                if (line[2].contains("#")) {
                    movImm8(line[1], line[2]);
                } else {
                    lslImm5(line[1], line[2], "#0");
                }
                break;
            case "cmp":
                if (line[2].contains("#")) {
                    cmpImm8(line[1], line[2]);
                } else {
                    cmpReg(line[1], line[2]);
                }
                break;
            case "ands":
                andReg(line[1], line[2]);
                break;
            case "eors":
                eorReg(line[1], line[2]);
                break;
            case "adcs":
                adcReg(line[1], line[2]);
                break;
            case "sbcs":
                sbcReg(line[1], line[2]);
                break;
            case "rors":
                rorReg(line[1], line[2]);
                break;
            case "tst":
                tstReg(line[1], line[2]);
                break;
            case "rsbs":
                rsbImm(line[1], line[2]);
                break;
            case "cmn":
                cmnReg(line[1], line[2]);
                break;
            case "orrs":
                orrReg(line[1], line[2]);
                break;
            case "muls":
                if (line[1].equals(line[3])) mul(line[1], line[2]);
                break;
            case "bics":
                bicReg(line[1], line[2]);
                break;
            case "mvns":
                mvnReg(line[1], line[2]);
                break;
            case "str", "ldr":
                if (line[2].equals("sp")) {
                    String immediateValue = (line.length == 3) ? "#0" : line[3];
                    if (line[0].equals("str")) {
                        strImm(line[1], immediateValue);
                    } else {
                        ldrImm(line[1], immediateValue);
                    }
                }
                break;
            case "add", "sub":
                if (line[1].equals("sp")) {
                    String immediateValue = line[2].contains("#") ? line[2] : line[3];
                    if (line[0].equals("add")) {
                        addSpImm(immediateValue);
                    } else {
                        subSpImm(immediateValue);
                    }
                }
                break;
            default:
                if (line[0].startsWith("b")) {
                    String branchLabel = line[1];
                    if (line[0].equals("b")) {
                        unconditionalBranch(branchLabel);
                    } else {
                        conditionalBranch(line[0], branchLabel);
                    }
                }
                break;
        }
    }

    private String bin2Hex(String binary) {
        int decimal = Integer.parseInt(binary, 2);
        return String.format("%04x", decimal);
    }

    private String intToBinaryString(String string, int bits) {
        if (string.charAt(0) == '-') {
            return binCompl2(intToBinaryString(string.substring(1), bits));
        } else {
            if (string.charAt(0) == 'r' || string.charAt(0) == '#') {
                string = string.substring(1);
            }
            StringBuilder builder = new StringBuilder(Integer.toBinaryString(Integer.parseInt(string)));
            while (builder.length() < bits) {
                builder.insert(0, "0");
            }
            return builder.toString();
        }
    }

    public String binCompl2(String bin) {
        StringBuilder onePunchMan = new StringBuilder(bin.replace('0', '2').replace('1', '0').replace('2', '1'));
        StringBuilder builder = new StringBuilder(onePunchMan.toString());
        boolean bool = false;
        for (int i = onePunchMan.length() - 1; i > 0; i--) {
            if (onePunchMan.charAt(i) == '1') {
                builder.setCharAt(i, '0');
            } else {
                builder.setCharAt(i, '1');
                bool = true;
                break;
            }
        }
        if (!bool) builder.insert(0, '1');
        return builder.toString();
    }

    private String binaryBy4(String strInt, int bits) {
        return intToBinaryString("#" + (Integer.parseInt(strInt.substring(1)) / 4), bits);
    }

    private void lslImm5(String rd, String rm, String imm5) {
        stringBuilder.append(bin2Hex("00000" + intToBinaryString(imm5, 5) + intToBinaryString(rm, 3) + intToBinaryString(rd, 3))).append(" ");
    }

    private void lsrImm5(String rd, String rm, String imm5) {
        stringBuilder.append(bin2Hex("00001" + intToBinaryString(imm5, 5) + intToBinaryString(rm, 3) + intToBinaryString(rd, 3))).append(" ");
    }

    private void asrImm5(String rd, String rm, String imm5) {
        stringBuilder.append(bin2Hex("00010" + intToBinaryString(imm5, 5) + intToBinaryString(rm, 3) + intToBinaryString(rd, 3))).append(" ");
    }

    private void addReg(String rd, String rn, String rm) {
        stringBuilder.append(bin2Hex("0001100" + intToBinaryString(rm, 3) + intToBinaryString(rn, 3) + intToBinaryString(rd, 3))).append(" ");
    }

    private void subReg(String rd, String rn, String rm) {
        stringBuilder.append(bin2Hex("0001101" + intToBinaryString(rm, 3) + intToBinaryString(rn, 3) + intToBinaryString(rd, 3))).append(" ");
    }

    private void addImm3(String rd, String rn, String imm3) {
        stringBuilder.append(bin2Hex("0001110" + intToBinaryString(imm3, 3) + intToBinaryString(rn, 3) + intToBinaryString(rd, 3))).append(" ");
    }

    private void subImm3(String rd, String rn, String imm3) {
        stringBuilder.append(bin2Hex("0001111" + intToBinaryString(imm3, 3) + intToBinaryString(rn, 3) + intToBinaryString(rd, 3))).append(" ");
    }

    private void movImm8(String rd, String imm8) {
        stringBuilder.append(bin2Hex("00100" + intToBinaryString(rd, 3) + intToBinaryString(imm8, 8))).append(" ");
    }

    private void cmpImm8(String rd, String imm8) {
        stringBuilder.append(bin2Hex("00101" + intToBinaryString(rd, 3) + intToBinaryString(imm8, 8))).append(" ");
    }

    private void addImm8(String rdn, String imm8) {
        stringBuilder.append(bin2Hex("00110" + intToBinaryString(rdn, 3) + intToBinaryString(imm8, 8))).append(" ");
    }

    private void subImm8(String rdn, String imm8) {
        stringBuilder.append(bin2Hex("00111" + intToBinaryString(rdn, 3) + intToBinaryString(imm8, 8))).append(" ");
    }

    private void andReg(String rdn, String rm) {
        stringBuilder.append(bin2Hex("0100000000" + intToBinaryString(rm, 3) + intToBinaryString(rdn, 3))).append(" ");
    }

    private void eorReg(String rdn, String rm) {
        stringBuilder.append(bin2Hex("0100000001" + intToBinaryString(rm, 3) + intToBinaryString(rdn, 3))).append(" ");
    }

    private void lslReg(String rdn, String rm) {
        stringBuilder.append(bin2Hex("0100000010" + intToBinaryString(rm, 3) + intToBinaryString(rdn, 3))).append(" ");
    }

    private void lsrReg(String rdn, String rm) {
        stringBuilder.append(bin2Hex("0100000011" + intToBinaryString(rm, 3) + intToBinaryString(rdn, 3))).append(" ");
    }

    private void asrReg(String rdn, String rm) {
        stringBuilder.append(bin2Hex("0100000100" + intToBinaryString(rm, 3) + intToBinaryString(rdn, 3))).append(" ");
    }

    private void adcReg(String rdn, String rm) {
        stringBuilder.append(bin2Hex("0100000101" + intToBinaryString(rm, 3) + intToBinaryString(rdn, 3))).append(" ");
    }

    private void sbcReg(String rdn, String rm) {
        stringBuilder.append(bin2Hex("0100000110" + intToBinaryString(rm, 3) + intToBinaryString(rdn, 3))).append(" ");
    }

    private void rorReg(String rdn, String rm) {
        stringBuilder.append(bin2Hex("0100000111" + intToBinaryString(rm, 3) + intToBinaryString(rdn, 3))).append(" ");
    }

    private void tstReg(String rn, String rm) {
        stringBuilder.append(bin2Hex("0100001000" + intToBinaryString(rm, 3) + intToBinaryString(rn, 3))).append(" ");
    }

    private void rsbImm(String rd, String rn) {
        stringBuilder.append(bin2Hex("0100001001" + intToBinaryString(rn, 3) + intToBinaryString(rd, 3))).append(" ");
    }

    private void cmpReg(String rn, String rm) {
        stringBuilder.append(bin2Hex("0100001010" + intToBinaryString(rm, 3) + intToBinaryString(rn, 3))).append(" ");
    }

    private void cmnReg(String rn, String rm) {
        stringBuilder.append(bin2Hex("0100001011" + intToBinaryString(rm, 3) + intToBinaryString(rn, 3))).append(" ");
    }

    private void orrReg(String rdn, String rm) {
        stringBuilder.append(bin2Hex("0100001100" + intToBinaryString(rm, 3) + intToBinaryString(rdn, 3))).append(" ");
    }

    private void mul(String rdm, String rn) {
        stringBuilder.append(bin2Hex("0100001101" + intToBinaryString(rn, 3) + intToBinaryString(rdm, 3))).append(" ");
    }

    private void bicReg(String rdn, String rm) {
        stringBuilder.append(bin2Hex("0100001110" + intToBinaryString(rm, 3) + intToBinaryString(rdn, 3))).append(" ");
    }

    private void mvnReg(String rd, String rm) {
        stringBuilder.append(bin2Hex("0100001111" + intToBinaryString(rm, 3) + intToBinaryString(rd, 3))).append(" ");
    }

    private void strImm(String rt, String offset) {
        stringBuilder.append(bin2Hex("10010" + intToBinaryString(rt, 3) + binaryBy4(offset, 8))).append(" ");
    }

    private void ldrImm(String rt, String offset) {
        stringBuilder.append(bin2Hex("10011" + intToBinaryString(rt, 3) + binaryBy4(offset, 8))).append(" ");
    }

    private void addSpImm(String offset) {
        stringBuilder.append(bin2Hex("101100000" + binaryBy4(offset, 7))).append(" ");
    }

    private void subSpImm(String offset) {
        stringBuilder.append(bin2Hex("101100001" + binaryBy4(offset, 7))).append(" ");
    }

    private void conditionalBranch(String condition, String label) {
        Map<String, String> conditionMap = Map.ofEntries(Map.entry("eq", "0000"), Map.entry("ne", "0001"), Map.entry("cs", "0010"), Map.entry("hs", "0010"), Map.entry("cc", "0011"), Map.entry("lo", "0011"), Map.entry("mi", "0100"), Map.entry("pl", "0101"), Map.entry("vs", "0110"), Map.entry("vc", "0111"), Map.entry("hi", "1000"), Map.entry("ls", "1001"), Map.entry("ge", "1010"), Map.entry("lt", "1011"), Map.entry("gt", "1100"), Map.entry("le", "1101"), Map.entry("al", "1110"));

        String binary = "1101" + conditionMap.getOrDefault(condition.substring(1), "1111");
        int labelIndex = labels.get(label + ":");
        int diff = labelIndex - instructionCount - 3;
        binary += intToBinaryString("" + diff, 8);
        stringBuilder.append(bin2Hex(binary)).append(" ");
    }

    private void unconditionalBranch(String label) {
        stringBuilder.append(bin2Hex("11100" + intToBinaryString(String.valueOf(labels.get(label + ":") - instructionCount - 3), 11))).append(" ");
    }
}
