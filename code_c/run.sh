# This little script run the generation of the bin for all the .s file (the given examples are kept in a _origin)
the_jar_file=$1
findalltheS=$(find . -name "*.s")

for the_s_file in $findalltheS
do
    java -jar $the_jar_file $the_s_file
done
 
# This is just to compare with the original version (but even if fails didn't mean that it did not work in logisim)
allnormalbin=$(find . -name "*.bin")
goodcounter=0
for normalbin in $allnormalbin
do
if [[ $normalbin != *_origin.bin ]]
then
    originbin=${normalbin%.*}_origin.bin
    sed -e '$a\' $normalbin > $normalbin.tmp
    diff $originbin $normalbin.tmp
    if [ $? -eq 0 ]
    then
        goodcounter=$((goodcounter+1))
    fi
    rm $normalbin.tmp
fi
done

echo "$goodcounter tests passed out of $(find . -name "*.s" | wc -l)"

