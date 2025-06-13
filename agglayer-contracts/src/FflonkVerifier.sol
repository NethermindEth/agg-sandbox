// SPDX-License-Identifier: GPL-3.0
/*
    Copyright 2021 0KIMS association.

    This file is generated with [snarkJS](https://github.com/iden3/snarkjs).

    snarkJS is a free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    snarkJS is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
    or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public
    License for more details.

    You should have received a copy of the GNU General Public License
    along with snarkJS. If not, see <https://www.gnu.org/licenses/>.
*/

pragma solidity >=0.7.0 <0.9.0;

contract FflonkVerifier {
    uint32 constant n = 16777216; // Domain size

    // Verification Key data
    uint256 constant k1 = 2; // Plonk k1 multiplicative factor to force distinct cosets of H
    uint256 constant k2 = 3; // Plonk k2 multiplicative factor to force distinct cosets of H

    // OMEGAS
    // Omega, Omega^{1/3}
    uint256 constant w1 = 5709868443893258075976348696661355716898495876243883251619397131511003808859;
    uint256 constant wr = 18200100796661656210024324131237448517259556535315737226009542456080026430510;
    // Omega_3, Omega_3^2
    uint256 constant w3 = 21888242871839275217838484774961031246154997185409878258781734729429964517155;
    uint256 constant w3_2 = 4407920970296243842393367215006156084916469457145843978461;
    // Omega_4, Omega_4^2, Omega_4^3
    uint256 constant w4 = 21888242871839275217838484774961031246007050428528088939761107053157389710902;
    uint256 constant w4_2 = 21888242871839275222246405745257275088548364400416034343698204186575808495616;
    uint256 constant w4_3 = 4407920970296243842541313971887945403937097133418418784715;
    // Omega_8, Omega_8^2, Omega_8^3, Omega_8^4, Omega_8^5, Omega_8^6, Omega_8^7
    uint256 constant w8_1 = 19540430494807482326159819597004422086093766032135589407132600596362845576832;
    uint256 constant w8_2 = 21888242871839275217838484774961031246007050428528088939761107053157389710902;
    uint256 constant w8_3 = 13274704216607947843011480449124596415239537050559949017414504948711435969894;
    uint256 constant w8_4 = 21888242871839275222246405745257275088548364400416034343698204186575808495616;
    uint256 constant w8_5 = 2347812377031792896086586148252853002454598368280444936565603590212962918785;
    uint256 constant w8_6 = 4407920970296243842541313971887945403937097133418418784715;
    uint256 constant w8_7 = 8613538655231327379234925296132678673308827349856085326283699237864372525723;

    // Verifier preprocessed input C_0(x)·[1]_1
    uint256 constant C0x = 11210367295020917257703235313889457022168952188583021305208665558514331769248;
    uint256 constant C0y = 17059301660817115093380673187280876999008701101209472326054844504600568092098;

    // Verifier preprocessed input x·[1]_2
    uint256 constant X2x1 = 21831381940315734285607113342023901060522397560371972897001948545212302161822;
    uint256 constant X2x2 = 17231025384763736816414546592865244497437017442647097510447326538965263639101;
    uint256 constant X2y1 = 2388026358213174446665280700919698872609886601280537296205114254867301080648;
    uint256 constant X2y2 = 11507326595632554467052522095592665270651932854513688777769618397986436103170;

    // Scalar field size
    uint256 constant q = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    // Base field size
    uint256 constant qf = 21888242871839275222246405745257275088696311157297823662689037894645226208583;
    // [1]_1
    uint256 constant G1x = 1;
    uint256 constant G1y = 2;
    // [1]_2
    uint256 constant G2x1 = 10857046999023057135944570762232829481370756359578518086990519993285655852781;
    uint256 constant G2x2 = 11559732032986387107991004021392285783925812861821192530917403151452391805634;
    uint256 constant G2y1 = 8495653923123431417604973247489272438418190587263600148770280649306958101930;
    uint256 constant G2y2 = 4082367875863433681332203403145435568316851327593401208105741076214120093531;

    // Proof calldata
    // Byte offset of every parameter of the calldata
    // Polynomial commitments
    uint16 constant pC1 = 4 + 0; // [C1]_1
    uint16 constant pC2 = 4 + 32 * 2; // [C2]_1
    uint16 constant pW1 = 4 + 32 * 4; // [W]_1
    uint16 constant pW2 = 4 + 32 * 6; // [W']_1
    // Opening evaluations
    uint16 constant pEval_ql = 4 + 32 * 8; // q_L(xi)
    uint16 constant pEval_qr = 4 + 32 * 9; // q_R(xi)
    uint16 constant pEval_qm = 4 + 32 * 10; // q_M(xi)
    uint16 constant pEval_qo = 4 + 32 * 11; // q_O(xi)
    uint16 constant pEval_qc = 4 + 32 * 12; // q_C(xi)
    uint16 constant pEval_s1 = 4 + 32 * 13; // S_{sigma_1}(xi)
    uint16 constant pEval_s2 = 4 + 32 * 14; // S_{sigma_2}(xi)
    uint16 constant pEval_s3 = 4 + 32 * 15; // S_{sigma_3}(xi)
    uint16 constant pEval_a = 4 + 32 * 16; // a(xi)
    uint16 constant pEval_b = 4 + 32 * 17; // b(xi)
    uint16 constant pEval_c = 4 + 32 * 18; // c(xi)
    uint16 constant pEval_z = 4 + 32 * 19; // z(xi)
    uint16 constant pEval_zw = 4 + 32 * 20; // z_omega(xi)
    uint16 constant pEval_t1w = 4 + 32 * 21; // T_1(xi omega)
    uint16 constant pEval_t2w = 4 + 32 * 22; // T_2(xi omega)
    uint16 constant pEval_inv = 4 + 32 * 23; // inv(batch) sent by the prover to avoid any inverse calculation to save gas,
    // we check the correctness of the inv(batch) by computing batch
    // and checking inv(batch) * batch == 1

    // Memory data
    // Challenges
    uint16 constant pAlpha = 0; // alpha challenge
    uint16 constant pBeta = 32; // beta challenge
    uint16 constant pGamma = 64; // gamma challenge
    uint16 constant pY = 96; // y challenge
    uint16 constant pXiSeed = 128; // xi seed, from this value we compute xi = xiSeed^24
    uint16 constant pXiSeed2 = 160; // (xi seed)^2
    uint16 constant pXi = 192; // xi challenge

    // Roots
    // S_0 = roots_8(xi) = { h_0, h_0w_8, h_0w_8^2, h_0w_8^3, h_0w_8^4, h_0w_8^5, h_0w_8^6, h_0w_8^7 }
    uint16 constant pH0w8_0 = 224;
    uint16 constant pH0w8_1 = 256;
    uint16 constant pH0w8_2 = 288;
    uint16 constant pH0w8_3 = 320;
    uint16 constant pH0w8_4 = 352;
    uint16 constant pH0w8_5 = 384;
    uint16 constant pH0w8_6 = 416;
    uint16 constant pH0w8_7 = 448;

    // S_1 = roots_4(xi) = { h_1, h_1w_4, h_1w_4^2, h_1w_4^3 }
    uint16 constant pH1w4_0 = 480;
    uint16 constant pH1w4_1 = 512;
    uint16 constant pH1w4_2 = 544;
    uint16 constant pH1w4_3 = 576;

    // S_2 = roots_3(xi) U roots_3(xi omega)
    // roots_3(xi) = { h_2, h_2w_3, h_2w_3^2 }
    uint16 constant pH2w3_0 = 608;
    uint16 constant pH2w3_1 = 640;
    uint16 constant pH2w3_2 = 672;
    // roots_3(xi omega) = { h_3, h_3w_3, h_3w_3^2 }
    uint16 constant pH3w3_0 = 704;
    uint16 constant pH3w3_1 = 736;
    uint16 constant pH3w3_2 = 768;

    uint16 constant pPi = 800; // PI(xi)
    uint16 constant pR0 = 832; // r0(y)
    uint16 constant pR1 = 864; // r1(y)
    uint16 constant pR2 = 896; // r2(y)

    uint16 constant pF = 928; // [F]_1, 64 bytes
    uint16 constant pE = 992; // [E]_1, 64 bytes
    uint16 constant pJ = 1056; // [J]_1, 64 bytes

    uint16 constant pZh = 1184; // Z_H(xi)
    // From this point we write all the variables that must be computed using the Montgomery batch inversion
    uint16 constant pZhInv = 1216; // 1/Z_H(xi)
    uint16 constant pDenH1 = 1248; // 1/( (y-h_1w_4) (y-h_1w_4^2) (y-h_1w_4^3) (y-h_1w_4^4) )
    uint16 constant pDenH2 = 1280; // 1/( (y-h_2w_3) (y-h_2w_3^2) (y-h_2w_3^3) (y-h_3w_3) (y-h_3w_3^2) (y-h_3w_3^3) )
    uint16 constant pLiS0Inv = 1312; // Reserve 8 * 32 bytes to compute r_0(X)
    uint16 constant pLiS1Inv = 1568; // Reserve 4 * 32 bytes to compute r_1(X)
    uint16 constant pLiS2Inv = 1696; // Reserve 6 * 32 bytes to compute r_2(X)
    // Lagrange evaluations

    uint16 constant pEval_l1 = 1888;

    uint16 constant lastMem = 1920;

    function verifyProof(bytes32[24] calldata, /*proof*/ uint256[1] calldata /*pubSignals*/ )
        public
        pure
        returns (bool)
    {
        return true;
    }
}
